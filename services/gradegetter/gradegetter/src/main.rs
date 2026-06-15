use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Router,
};
use backend_common::gradegetter::GradesHashMap;
use crypto_utils::{decrypt_string, encrypt_string};
use futures_util::{SinkExt, StreamExt};
use regex::Regex;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    collections::{BTreeMap, HashMap},
    process::Stdio,
    str,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpListener,
    process::Command,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};
use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest};
use tracing::{debug, error, info, info_span, instrument, trace, warn, Instrument};
use uuid::Uuid;

use crate::secrets::SECRETS;
mod secrets;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let _guard = backend_common::tracing::init_tracing();

    let database_string = &SECRETS.database_url;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_string)
        .await
        .context("failed to connect to database")?;

    let pool = Arc::new(pool);

    // Token Getter Thread?
    let pool_token = Arc::clone(&pool);
    setup_token_thread(pool_token);

    // Grade fetcher
    let pool_grades = Arc::clone(&pool);
    setup_grade_thread(pool_grades);

    let pool_axum = Arc::clone(&pool);
    let app = Router::new().route("/health", get(health)).route(
        "/userinit",
        post({
            let pool_axum = Arc::clone(&pool_axum);
            move |uuid: String| {
                let pool = Arc::clone(&pool_axum);

                async move { user_token_initalize(pool, uuid).await }
            }
        }),
    );

    let listener = TcpListener::bind("[::]:3001").await.unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

fn setup_token_thread(pool_token: Arc<PgPool>) {
    tokio::spawn(async move {
        loop {
            let batch_token_span = info_span!("batch_tokengetter_job");
            let db_span = info_span!(parent: &batch_token_span, "tokengetter_grab_userinfo_query");
            if let Ok(users) =
                sqlx::query!("SELECT id, encrypted_email, encrypted_password FROM schoology_auth")
                    .fetch_all(&*pool_token)
                    .instrument(db_span)
                    .await
                    .map_err(|err| {
                        error!(error = %err, "[Database failure]: failed to look up users in the tokengetter thread");
                    })
            {
                for user in users {
                    let per_user_span = info_span!(parent: &batch_token_span, "fetch_user_token", user.id = %user.id);
                    let pool_token_clone = pool_token.clone();

                    let (id, email, password) =
                        (user.id, user.encrypted_email, user.encrypted_password);

                    async move {
                        let Ok(dec_password) = decrypt_string(password) else {
                            error!(
                                user.id = %id,
                                "[Decryption Error]: decrypting password failed, (token thread) skipping user due to decryption error",
                            );

                            return;
                        };

                        let Ok(dec_email) = decrypt_string(email) else {
                            error!(
                                user.id = %id,
                                "[Decryption Error]: decrypting email failed, (token thread) skipping user due to decryption error",
                            );

                            return;
                        };

                        let get_token_span = info_span!("get_token_func");

                        let Ok(token) = get_token(dec_email.as_str(), dec_password.as_str(), id, false).instrument(get_token_span)
                            .await else {
                                error!(
                                    user.id = %id,
                                    "[Decryption Error]: decrypting email failed, (token thread) skipping user due to decryption error",
                                );

                                let db_span = info_span!("set_token_null_query");

                                let _ = sqlx::query!(
                                    "UPDATE schoology_auth SET session_token = NULL WHERE id = $1",
                                    id
                                )
                                .execute(&*pool_token_clone)
                                .instrument(db_span)
                                .await
                                .map_err(|err| {
                                    error!(error = %err, "[Database failure]: (inside token thread); failed to set user session_token to null");
                                });

                                return;
                            };

                        let db_span = info_span!("set_new_token_query");

                        let _ = sqlx::query!(
                            "UPDATE schoology_auth SET session_token = $1 WHERE id = $2",
                            token,
                            id
                        )
                        .execute(&*pool_token_clone)
                        .instrument(db_span)
                        .await
                        .map_err(|err| {
                            error!(error = %err, "[Database failure]: failed to update session token");
                        });

                        info!(
                            action = "gradegetter.update_token",
                            user.id = %user.id,
                            "[INFO]: Updated Token for a user!"
                        );

                    }
                    .instrument(per_user_span)
                    .await;
                }
            }
            drop(batch_token_span);
            tokio::time::sleep(std::time::Duration::from_secs(1800)).await // 30 minutes
        }
    });
}

fn setup_grade_thread(pool_grades: Arc<PgPool>) {
    tokio::spawn(async move {
        loop {
            let batch_grade_span = info_span!("batch_fetch_user_grades");
            let db_span = info_span!(parent: &batch_grade_span, "gradegetter_grab_userinfo_query");
            if let Ok(users) = sqlx::query!("SELECT id, session_token FROM schoology_auth")
                .fetch_all(&*pool_grades)
                .instrument(db_span)
                .await
                .map_err(|err| {
                    error!(error = %err, "[Database failure]: failed to look up users in the gradegetter thread");
                })
            {
                for user in users {
                    if let (id, Some(token)) = (user.id, user.session_token) {
                        let per_user_span = info_span!(parent: &batch_grade_span, "fetch_user_grades", user.id = %id);
                        let pool_grades_clone = pool_grades.clone();

                        async move {
                            let Ok(token) = decrypt_string(token) else {
                                error!(
                                    "[ERROR]: decyrpt_string, (grade thread) skipping user due to decryption error:  {}",
                                    id
                                );

                                let db_span = info_span!("gradegetter_set_token_null_query");

                                let _ = sqlx::query!(
                                    "UPDATE schoology_auth SET session_token = NULL WHERE id = $1",
                                    id
                                )
                                .execute(&*pool_grades_clone)
                                .instrument(db_span)
                                .await
                                .map_err(|err| {
                                    error!(error = %err, "[Database failure]: (inside grade thread); failed to set user session_token to null ");
                                });

                                return;
                            };

                            match fetch_grades(token, id).await {
                                Ok(grades_json) => {
                                    let db_span = info_span!("update_grades_query");

                                    let _ = sqlx::query!(
                                    "INSERT INTO grades (id, grades) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET grades = EXCLUDED.grades",
                                        id, grades_json
                                    )
                                    .execute(&*pool_grades_clone)
                                    .instrument(db_span)
                                    .await
                                    .map_err(|err| {
                                        error!(error = %err, "[Database failure]: (inside grade thread); failed to update user grades");
                                    });

                                    notify_updated_grades(id).await;

                                    info!(
                                        action = "gradegetter.update_grades",
                                        user.id = %user.id,
                                        "[INFO]: Updated Grades for a user!"
                                    );

                                }
                                Err(e) => {
                                    error!(error = %e, "failed to fetch grades for user");
                                    tokio::time::sleep(Duration::from_secs(10)).await;
                                }
                            }
                        }
                        .instrument(per_user_span)
                        .await;
                    }
                }
            }
            drop(batch_grade_span);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await
        }
    });
}

async fn shutdown_signal() {
    let ctrl_c = signal::ctrl_c();

    let terminte = async {
        signal(SignalKind::terminate())
            .expect("failed to install the SIGTERM handler 🥲")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminte => {},

    }

    info!("Signal recvived now starting graceful shutdown");
}

async fn health() -> Result<(), axum::http::StatusCode> {
    Ok(())
}

#[instrument(
    name = "user_init",
    skip(pool),
    fields(
        user.id = %uuid,
    )
)]
async fn user_token_initalize(
    pool: Arc<PgPool>,
    uuid: String,
) -> Result<String, axum::http::StatusCode> {
    let uuid = uuid::Uuid::parse_str(&uuid).map_err(|err| {
        error!(error = %err, "ERROR ENCODING UUID");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let userinit_userdata_db_span = info_span!("get_user_data_query");
    let user = sqlx::query!(
           "SELECT id, encrypted_email, encrypted_password FROM schoology_auth WHERE session_token IS NULL AND id = $1",
            uuid
        )
        .fetch_one(&*pool)
        .instrument(userinit_userdata_db_span)
        .await
        .map_err(|err| {
            error!(error = %err, "[Database failure]: failed to grab user info during user initalization");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        });

    let user = match user {
        Ok(user) => user,
        Err(_) => return Err(axum::http::StatusCode::UNAUTHORIZED),
    };

    let (id, email, password) = (user.id, user.encrypted_email, user.encrypted_password);

    let dec_password = match decrypt_string(password) {
        Ok(dec_password) => dec_password,
        Err(err) => {
            error!(error = %err, "[Decryption Error]: failed to decrypt password");
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let dec_email = match decrypt_string(email) {
        Ok(dec_email) => dec_email,
        Err(err) => {
            error!(error = %err, "[Decryption Error]: failed to decrypt email");
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let token = match get_token(dec_email.as_str(), dec_password.as_str(), id, true).await {
        Ok(token) => token,
        Err(err) => {
            error!(error = %err, "[TokenGetter failure]: get_token failure during user init");
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }
    };

    let update_token_db_span = info_span!("update_token_query");
    sqlx::query!(
        "UPDATE schoology_auth SET session_token = $1 WHERE id = $2",
        token,
        id
    )
    .execute(&*pool)
    .instrument(update_token_db_span)
    .await
    .map_err(|err| {
        error!(error = %err, "[Database failure]: failed to set user session_token during user initalization");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!(
        action = "gradegetter.update_token",
        user.id = %user.id,
        "[INFO]: Update token for a user during user_init!"
    );

    let Ok(token) = decrypt_string(token) else {
        error!(
            user.id = %id,
            "[Decryption Error]: decrypting token failed, (user_init) skipping user due to decryption error",
        );

        let db_span = info_span!("set_token_null_query");

        let _ = sqlx::query!(
            "UPDATE schoology_auth SET session_token = NULL WHERE id = $1",
            id
        )
        .execute(&*pool)
        .instrument(db_span)
        .await
        .map_err(|err| {
            error!(error = %err, "[Database failure]: failed to set user session_token to null during user initalization");
        });

        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    };

    let update_grades_db_span = info_span!("update_grades_db_span");

    sqlx::query!("INSERT INTO grades (id, grades) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET grades = EXCLUDED.grades",
        id,
        fetch_grades(token, id)
            .await.map_err(|err| {
                error!(error = %err, "[GradeGetter fetch failure]: failed fetch grades while user initalization");
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            })?,
        )
        .execute(&*pool)
        .instrument(update_grades_db_span)
        .await
        .map_err(|err| {
            error!(error = %err, "[Database failure]: failed to set grades in db while user initalization");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        action = "gradegetter.update_grade",
        user.id = %user.id,
        "[INFO]: Update grades for a user during user_init!"
    );

    Ok("Hi".to_string())
}

#[derive(Serialize, Debug)]
pub struct ForwardMessage {
    id: Uuid,
    status: String,
}

#[instrument(
    name = "get_token",
    skip(email, password),
    fields(
        user.id = %user_id,
        user.init = %user_init,
    )
)]
async fn get_token(
    email: &str,
    password: &str,
    user_id: Uuid,
    user_init: bool,
) -> Result<Vec<u8>, anyhow::Error> {
    let executable =
        dotenvy::var("PUPPETEER_EXECUTABLE_PATH").expect("PUPPETEER_EXECUTABLE_PATH not found");
    let mut child = Command::new("node")
        .env("PUPPETEER_EXECUTABLE_PATH", executable)
        .arg("../tokengetter/") // ASSuming this is the path; adjust if needed...ass
        .arg(email)
        .arg(password)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to execute tokengetter")?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture stderr"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture stderr"))?;
    let mut lines = BufReader::new(stderr).lines();

    if user_init {
        let mut request =
            "ws://gradegetter_backend:3002/internal/forward_ws".into_client_request()?;

        let internal_api = &SECRETS.internal_api_key;

        request
            .headers_mut()
            .insert("Authorization", format!("Basic {}", internal_api).parse()?);

        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, _read) = ws_stream.split();

        tokio::spawn(async move {
            /*HACK: unwarappy */
            while let Some(line) = lines.next_line().await.unwrap() {
                let msg = ForwardMessage {
                    id: user_id,
                    status: line,
                };

                let json = serde_json::to_string(&msg).unwrap(); //HACK: unwrap
                write
                    .send(tokio_tungstenite::tungstenite::Message::Text(json.into()))
                    .await
                    .unwrap(); //HACK: unwrap
            }
        });
    }

    let mut token = String::new();

    child
        .wait()
        .await
        .context("failed to execute tokengetter")?;

    stdout
        .read_to_string(&mut token)
        .await
        .context("failed to insert stdout contents into token string")?;

    let token = token.trim().to_string();

    if token.is_empty() {
        warn!(
            action = "gradegetter.get_token",
            user.id = %user_id,
            "TokenGetter returned an empty token"
        );
        anyhow::bail!("tokengetter returned an empty token");
    }

    if !token.starts_with("SESS") {
        warn!(
            action = "gradegetter.get_token",
            user.id = %user_id,
            "TokenGetter returned invalid data"
        );
        anyhow::bail!(
            "tokengetter returned invalid session token format: {}",
            token
        );
    }

    let encrypted_token = match encrypt_string(&token) {
        Ok(encrypted_token) => encrypted_token,
        Err(err) => {
            error!(error = %err, "[ERROR]: failed to encrypt token inside gradegetter");
            anyhow::bail!("token encryption failed");
        }
    };

    Ok(encrypted_token)
}

#[instrument(
    name = "fetch_grades",
    skip(token),
    fields(
        user.id = %user_id,
    )
)]
async fn fetch_grades(token: String, user_id: Uuid) -> Result<Vec<u8>, anyhow::Error> {
    if token.trim().is_empty() {
        warn!(
            action = "gradegetter.fetch_grades",
            user.id = %user_id,
        "somehow there is an empty token inside the fetch grades function, will be cleaned up next run"
        );

        return Err(anyhow::anyhow!("empty token bruh"));
    }

    let forms = select_grade_period(token.clone())
        .await
        .inspect_err(|err| {
            error!(error = %err, "select_grade_peroid failed");
        })
        .context("select_grade_period failed")?;

    let html = fetch_final_grades_export(
        forms.form_build_id.as_str(),
        forms.form_token.as_str(),
        forms.class_ids,
        token,
    )
    .await
    .context("fetch_final_grades_export failed")?;

    let grades: GradesHashMap = parse_grades_html(html).context("parse_grades_html failed")?;

    let grades = serde_json::to_value(grades)?.to_string();

    let encrypted_grades = match encrypt_string(&grades) {
        Ok(encrypted_grades) => encrypted_grades,
        Err(err) => {
            error!(error = %err, "token encryption failed");
            anyhow::bail!("grade encryption failed");
        }
    };

    Ok(encrypted_grades)
}

static FORM_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new(r#"name="form_build_id" id="([^"]+)""#).unwrap());

static FORM_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"<input type="hidden" name="form_token" id="edit-s-grades-export-form-form-token-1" value="([^"]+)""#).unwrap()
});

// used specifically for the select_grade_period function due to its slightly differnt html
static FORM_TOKEN_RE_GP: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new(r#"form-token" value="([^"]+)""#).unwrap());

static GRADING_PEROID_RE: LazyLock<Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"name="grading_period\[(\d+)\]".*?form-checkbox-title">.*?Q\d"#).unwrap()
});

static COURSE_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new(r#"courses\[(\d+)\]\[selected\]"#).unwrap());

#[derive(Debug)]
struct QuarterForms {
    form_build_id: String,
    form_token: String,
    grading_periods: HashMap<String, String>,
}

// this function gets the form token, form build id, and quarter information
#[instrument(name = "fetch_export_initial_form_data", skip(token), fields())]
async fn fetch_export_initial_form_data(token: String) -> Result<QuarterForms, anyhow::Error> {
    debug!("ENTERED FETCH EXPORT");

    let mut form_build_id = "N/A".to_string();
    let mut form_token = "N/A".to_string();

    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();

    debug!("before parse");

    headers.insert("Cookie", token.parse()?);

    debug!("after parse");

    let req = client
        .request(
            reqwest::Method::GET,
            "https://essexnorthshore.schoology.com/grades/export",
        )
        .headers(headers);

    let response = req.send().await.inspect_err(|err| {
        error!(error = ?err, "sending the request for fetch_export_initial_form_data failed");
    })?;
    let body = response.text().await?;

    if let Some(caps) = FORM_ID_RE.captures(body.as_str()) {
        form_build_id = caps[1].to_string();
        debug!("fetch_export_initial_form_data: form_build_id match found");
    } else {
        debug!("fetch_export_initial_form_data: form_build_id NO match found");
    }

    if let Some(caps) = FORM_TOKEN_RE.captures(body.as_str()) {
        form_token = caps[1].to_string();
        debug!("fetch_export_initial_form_data: form_token match found");
    } else {
        debug!("fetch_export_initial_form_data: form_token NO match found");
    }

    let mut grading_periods: HashMap<String, String> = HashMap::new();

    let quater_ids: Vec<String> = GRADING_PEROID_RE
        .captures_iter(&body)
        .map(|c| c.extract::<1>().1[0].to_string())
        .collect();

    // BUG: crash here possibly fixed...but just a warning
    for id in &quater_ids[quater_ids.len() - 4..] {
        trace!("quarter ids: {id}");
        grading_periods.insert(format!("grading_period[{}]", id), id.to_string());
    }

    let output = QuarterForms {
        form_build_id,
        form_token,
        grading_periods,
    };

    debug!("fetch_export_inital_form_data output: {:?}", output);

    Ok(output)
}

#[derive(Debug)]
struct ClassForms {
    form_build_id: String,
    form_token: String,
    class_ids: HashMap<String, String>,
}

// This inputs the form build, form id, and quarter values from last from the last function (fetch_export_initial_form_data)
// and selects the grading period
#[instrument(name = "select_grade_period", skip(token), fields())]
async fn select_grade_period(token: String) -> Result<ClassForms, anyhow::Error> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert("Cookie", token.parse()?);

    headers.insert("accept-language", "en-US,en;q=0.9".parse()?);
    headers.insert("cache-control", "max-age=0".parse()?);
    headers.insert("content-type", "application/x-www-form-urlencoded".parse()?);
    headers.insert("origin", "https://essexnorthshore.schoology.com".parse()?);
    headers.insert("priority", "u=0, i".parse()?);
    headers.insert(
        "referer",
        "https://essexnorthshore.schoology.com/grades/grades".parse()?,
    );
    headers.insert(
        "sec-ch-ua",
        "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\"".parse()?,
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse()?);
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse()?);
    headers.insert("sec-fetch-dest", "document".parse()?);
    headers.insert("sec-fetch-mode", "navigate".parse()?);
    headers.insert("sec-fetch-site", "same-origin".parse()?);
    headers.insert("sec-fetch-user", "?1".parse()?);
    headers.insert("upgrade-insecure-requests", "1".parse()?);

    let params_needed = fetch_export_initial_form_data(token)
        .await
        .context("fetch_export_form_tokens failed")?;

    let mut params = params_needed.grading_periods;

    params.insert("form_id".to_string(), "s_grades_export_form".to_string());
    params.insert("op".to_string(), "Next".to_string());
    params.insert("form_build_id".to_string(), params_needed.form_build_id);
    params.insert("form_token".to_string(), params_needed.form_token);

    let req = client
        .request(
            reqwest::Method::POST,
            "https://essexnorthshore.schoology.com/grades/export",
        )
        .headers(headers)
        .form(&params);

    let response = req.send().await?;
    let body = response.text().await?;

    let mut form_build_id = "N/A".to_string();
    let mut form_token = "N/A".to_string();

    if let Some(caps) = FORM_ID_RE.captures(body.as_str()) {
        form_build_id = caps[1].to_string();
        debug!("select_grade_period: form_build_id match found");
    } else {
        debug!("select_grade_period: form_build_id NO match found");
    }

    if let Some(caps) = FORM_TOKEN_RE_GP.captures(body.as_str()) {
        form_token = caps[1].to_string();
        debug!("select_grade_period: form_token match found");
    } else {
        debug!("select_grade_period: form_token NO match found");
    }

    let class_ids = fetch_class_ids(body)
        .await
        .context("fetch_class_ids failed: {}")?;

    let output = ClassForms {
        form_build_id,
        form_token,
        class_ids,
    };

    debug!("select_grade_period output: {:?}", output);

    Ok(output)
}

#[instrument(name = "fetch_class_ids", skip(body), fields())]
async fn fetch_class_ids(body: String) -> Result<HashMap<String, String>, anyhow::Error> {
    let mut hashmap: HashMap<String, String> = HashMap::new();

    for (_, [id]) in COURSE_ID_RE
        .captures_iter(body.as_str())
        .map(|c| c.extract())
    {
        trace!("class ids: {id}");
        hashmap.insert(format!("courses[{}][selected]", id), "1".to_string());
    }
    Ok(hashmap)
}

type ClassIdsHashMap = HashMap<String, String>;

// Selects classes and gets the final export, creates html file
#[instrument(
    name = "fetch_final_grades_export",
    skip(form_build_id, form_token, class_ids_hashmap, token),
    fields()
)]
async fn fetch_final_grades_export(
    form_build_id: &str,
    form_token: &str,
    class_ids_hashmap: ClassIdsHashMap,
    token: String,
) -> Result<String, anyhow::Error> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert("Cookie", token.parse()?);

    headers.insert("accept-language", "en-US,en;q=0.9".parse()?);
    headers.insert("cache-control", "max-age=0".parse()?);
    headers.insert("content-type", "application/x-www-form-urlencoded".parse()?);
    headers.insert("origin", "https://essexnorthshore.schoology.com".parse()?);
    headers.insert("priority", "u=0, i".parse()?);
    headers.insert(
        "referer",
        "https://essexnorthshore.schoology.com/grades/grades".parse()?,
    );
    headers.insert(
        "sec-ch-ua",
        "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\"".parse()?,
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse()?);
    headers.insert("sec-ch-ua-platform", "\"macOS\"".parse()?);
    headers.insert("sec-fetch-dest", "document".parse()?);
    headers.insert("sec-fetch-mode", "navigate".parse()?);
    headers.insert("sec-fetch-site", "same-origin".parse()?);
    headers.insert("sec-fetch-user", "?1".parse()?);
    headers.insert("upgrade-insecure-requests", "1".parse()?);

    let mut params = class_ids_hashmap;

    params.insert("form_id".to_string(), "s_grades_export_form".to_string());
    params.insert("form_build_id".to_string(), form_build_id.to_string());
    params.insert("form_token".to_string(), form_token.to_string());

    let req = client
        .request(
            reqwest::Method::POST,
            "https://essexnorthshore.schoology.com/grades/export",
        )
        .headers(headers)
        .form(&params);
    let response = req.send().await?;

    debug!("fetch_final_grades_export: page_url {}", response.url());

    let body = response.text().await?;

    Ok(body)
}

#[instrument(name = "parse_grades_html", skip(html), fields())]
fn parse_grades_html(html: String) -> Result<GradesHashMap, anyhow::Error> {
    let document = scraper::Html::parse_document(html.as_str());
    let grade_selector = scraper::Selector::parse("td.grade, td.grade.no-grade")
        .expect("could not parse grade_selector");

    let Ok(row_selector) = scraper::Selector::parse("tr") else {
        anyhow::bail!("could not parse row_selector");
    };

    let mut course_grades: GradesHashMap = BTreeMap::new();
    let mut current_course: Option<String> = None;

    for row in document.select(&row_selector) {
        if row.value().has_class(
            "course-title",
            scraper::CaseSensitivity::AsciiCaseInsensitive,
        ) {
            let title_text = row.text().collect::<String>().trim().to_string();
            let cleaned_title_text: String = match title_text.find("\u{a0}:\u{a0}") {
                Some(index) => title_text[..index].to_string(),
                None => title_text.to_string(),
            };

            if cleaned_title_text == "Class of 2028 Guidance" {
                continue;
            }

            current_course = Some(cleaned_title_text.clone());
            course_grades.insert(cleaned_title_text, Vec::new());
        } else if let Some(course) = &current_course {
            for grade_cell in row.select(&grade_selector) {
                if Some("grade final-grade") == grade_cell.attr("class") {
                    // dont add "final-grade" in the course_grades hashmap
                    continue;
                };

                let grade_text = grade_cell
                    .text()
                    .collect::<String>()
                    .trim()
                    .replace("%", "");
                let grade = grade_text.parse::<f32>().ok();
                course_grades.get_mut(course).unwrap().push(grade);
            }
        }
    }

    Ok(course_grades)
}

#[instrument(name = "notify_updated_grades", skip(), fields(user.id = %user_id))]
async fn notify_updated_grades(user_id: Uuid) {
    let message = serde_json::json!({
        "namespace": "gradegetter",
        "payload": "GradesUpdated"
    });

    let url = format!(
        "http://notification_backend:3003/internal/user_message/{}",
        user_id
    );

    let _ = reqwest::Client::new()
        .post(url)
        .header(
            "Authorization",
            format!("Basic {}", SECRETS.internal_api_key.as_str()),
        )
        .json(&message)
        .send()
        .await
        .map_err(|err| tracing::error!("failed to notify listing added: {}", err));
}
