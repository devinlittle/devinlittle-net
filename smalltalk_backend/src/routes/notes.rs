use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use common::{
    smalltalk::{
        NoteCreateRequest, NoteGroupCreateRequest, NotePatchRequest, NotesGroupPatchRequest,
        SmalltalkNote, SmalltalkNotesEvent, SmalltalkNotesGroup, SmalltalkNotesSyncResponse,
    },
    AuthenticatedUser,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::routes::AppState;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SyncParams {
    /// Unix timestamp in milliseconds
    #[schema(value_type = i64, example = 1715760000000_i64)]
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub since: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/notes/sync",
    params(("since" = i64 , Query, description = "unix timestamp in milliseconds to grab notes after")),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "lists notes and groups with an updated at time higher than the since param", body = Vec<SmalltalkNotesSyncResponse>),
        (status = 401, description = "Credentials Incorrect"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn note_sync(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(sync_params): Query<SyncParams>,
) -> Result<Json<SmalltalkNotesSyncResponse>, StatusCode> {
    let synced_notes = sqlx::query_as!(
        SmalltalkNote,
        r#"
    SELECT
        id,
        user_id,
        enc_name,
        enc_content,
        group_id,
        is_protected as "is_protected!",
        password_hash,
        salt,
        rank as "rank!",
        is_deleted as "is_deleted!",
        updated_at as "updated_at!: chrono::DateTime<chrono::Utc>",
        created_at as "created_at!: chrono::DateTime<chrono::Utc>"
    FROM smalltalk_notes
    WHERE user_id = $1
    AND updated_at > $2::timestamptz
    ORDER BY updated_at ASC
    "#,
        user.uuid,
        sync_params.since as _ // have the "as _" because rust-analayzer doesn't like this line
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|err| {
        tracing::error!(
            "Note sync database error when attempting to fetch notes: {:?}",
            err
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let synced_groups = sqlx::query_as!(
        SmalltalkNotesGroup,
        r#"
    SELECT
        id,
        user_id,
        enc_group_name,
        enc_group_metadata,
        rank as "rank!",
        is_deleted as "is_deleted!",
        updated_at as "updated_at!: chrono::DateTime<chrono::Utc>",
        created_at as "created_at!: chrono::DateTime<chrono::Utc>"
    FROM smalltalk_notes_groups
    WHERE user_id = $1
    AND updated_at > $2::timestamptz
    ORDER BY updated_at ASC
    "#,
        user.uuid,
        sync_params.since as _ // have the "as _" because rust-analayzer doesn't like this line
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|err| {
        tracing::error!(
            "Note sync database error when attempting to fetch groups: {:?}",
            err
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let output = SmalltalkNotesSyncResponse {
        notes: synced_notes,
        groups: synced_groups,
    };

    tracing::info!(
        "User '{0} ({1})' synced notes + groups",
        user.username,
        user.uuid
    );

    Ok(Json(output))
}

// maybe use this for regrabbing a forgotten note?
// /notes grab_notes GET

//async fn grab_notes(State(state): State<AppState>, )

#[utoipa::path(
    post,
    path = "/notes/note",
    request_body = NoteCreateRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "created note"),
        (status = 500, description = "internal server error"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn create_note(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<NoteCreateRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut rb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO smalltalk_notes (id, user_id, enc_name, is_protected, password_hash, salt, created_at, updated_at) VALUES ("
    );

    let mut separated = rb.separated(", ");

    separated.push("gen_random_uuid()"); // note id
    separated.push_bind(user.uuid);
    separated.push_bind(req.enc_name);

    separated.push_bind(req.is_protected);
    separated.push_bind(req.password_hash);
    separated.push_bind(req.salt);

    separated.push("NOW()"); // created_at
    separated.push("NOW()"); // updated_at

    rb.push(") RETURNING *");

    let query = rb.build_query_as::<SmalltalkNote>();
    let note = query.fetch_one(&state.pool).await.map_err(|err| {
        tracing::error!("Error creating note, {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .broadcast_note_event(user.uuid, SmalltalkNotesEvent::NoteAdded { note })
        .await;

    tracing::info!("User '{0} ({1})' created note", user.username, user.uuid);

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/notes/note/{note_id}",
    params(
        ("note_id", description = "the id of the specific note to soft delete")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "updates note state and broadcasts update on websocket"),
        (status = 500, description = "internal server error"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn soft_del_note(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(note_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let query = sqlx::query!(
        "UPDATE smalltalk_notes SET
            enc_content = NULL,
            is_deleted = TRUE
        WHERE id = $1 AND user_id = $2 RETURNING id",
        note_id,
        user.uuid
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        tracing::error!("Error deleting note: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .broadcast_note_event(
            user.uuid,
            SmalltalkNotesEvent::NoteDeleted { note_id: query.id },
        )
        .await;

    tracing::info!("User '{0} ({1})' deleted a note", user.username, user.uuid);

    Ok(StatusCode::OK)
}

#[utoipa::path(
    patch,
    path = "/notes/note/{note_id}",
    request_body = NotePatchRequest,
    params(
        ("note_id", description = "the id of the specific note to update")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "updates note and broadcasts update on websocket"),
        (status = 500, description = "internal server error"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn update_note(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(note_id): Path<Uuid>,
    Json(req): Json<NotePatchRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match handle_patch_note(&state.pool, &note_id, &user.uuid, req).await {
        Ok(note) => {
            let event = SmalltalkNotesEvent::NoteUpdated { note_id, note };

            state.broadcast_note_event(user.uuid, event).await;

            tracing::info!("User '{0} ({1})' updated a note", user.username, user.uuid);

            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to patch note: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_patch_note(
    pool: &sqlx::PgPool,
    note_id: &Uuid,
    user_id: &Uuid,
    req: NotePatchRequest,
) -> Result<SmalltalkNote, sqlx::Error> {
    let mut rb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE smalltalk_notes SET ");
    let mut first = true;

    macro_rules! set {
        ($col:literal, $value:expr) => {{
            if !first {
                rb.push(", ");
            }
            first = false;
            rb.push(concat!($col, " = ")).push_bind($value);
        }};
    }

    if let Some(name) = req.enc_name {
        set!("enc_name", name);
    }

    if let Some(content) = req.enc_content {
        set!("enc_content", content);
    } else if req.is_deleted == Some(true) {
        if !first {
            rb.push(", ");
        }
        first = false;
        rb.push("enc_content = NULL");
    }

    if let Some(protected) = req.is_protected {
        set!("is_protected", protected);
    }

    if let Some(hash) = req.password_hash {
        set!("password_hash", hash);
    }

    if let Some(salt) = req.salt {
        set!("salt", salt);
    }

    if let Some(rank) = req.rank {
        set!("rank", rank);
    }

    if let Some(deleted) = req.is_deleted {
        set!("is_deleted", deleted);
    }

    if !first {
        rb.push(", ");
    }
    rb.push("updated_at = NOW()");

    rb.push(" WHERE id = ")
        .push_bind(note_id)
        .push(" AND user_id = ")
        .push_bind(user_id)
        .push(" RETURNING *");

    let query = rb.build_query_as::<SmalltalkNote>();
    let updated_note = query.fetch_one(pool).await?;

    Ok(updated_note)
}

#[utoipa::path(
    post,
    path = "/notes/group",
    request_body = NoteGroupCreateRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "created note"),
        (status = 500, description = "internal server error"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn create_group(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<NoteGroupCreateRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut rb: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO smalltalk_notes_groups (id, user_id, enc_group_name, enc_group_metadata, updated_at, created_at) VALUES (");

    let mut separated = rb.separated(", ");

    separated.push("gen_random_uuid()"); // group id
    separated.push_bind(user.uuid);
    separated.push_bind(req.enc_group_name);
    separated.push_bind(req.enc_group_metadata);

    separated.push("NOW()"); // created_at
    separated.push("NOW()"); // updated_at

    rb.push(") RETURNING *");

    let query = rb.build_query_as::<SmalltalkNotesGroup>();
    let group = query.fetch_one(&state.pool).await.map_err(|err| {
        tracing::error!("Error creating note, {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .broadcast_note_event(user.uuid, SmalltalkNotesEvent::GroupCreated { group })
        .await;

    tracing::info!("User '{0} ({1})' created group", user.username, user.uuid);

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/notes/group/{group_id}",
    params(
        ("group_id", description = "the id of the specific group to soft delete")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "deletes group and broadcasts the state on websocket"),
        (status = 500, description = "internal server error"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn delete_group(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let query = sqlx::query!(
        "UPDATE smalltalk_notes_groups SET
            enc_group_metadata = NULL,
            is_deleted = TRUE
        WHERE id = $1 AND user_id = $2 RETURNING id",
        group_id,
        user.uuid
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|err| {
        tracing::error!("Error deleting group: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .broadcast_note_event(
            user.uuid,
            SmalltalkNotesEvent::GroupDeleted { group_id: query.id },
        )
        .await;

    tracing::info!("User '{0} ({1})' deleted a group", user.username, user.uuid);

    Ok(StatusCode::OK)
}

#[utoipa::path(
    patch,
    path = "/notes/group/{group_id}",
    request_body = NotesGroupPatchRequest,
    params(
        ("group_id", description = "the id of the specific group to update")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "updates group and broadcasts update on websocket"),
        (status = 500, description = "internal server error"),
    ),
    tag = "smalltalk_notes"
)]
pub async fn update_group(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(group_id): Path<Uuid>,
    Json(req): Json<NotesGroupPatchRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    match handle_patch_group(&state.pool, &group_id, &user.uuid, req).await {
        Ok(group) => {
            let event = SmalltalkNotesEvent::GroupUpdated { group_id, group };

            state.broadcast_note_event(user.uuid, event).await;

            tracing::info!("User '{0} ({1})' updated a group", user.username, user.uuid);

            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to patch group: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_patch_group(
    pool: &sqlx::PgPool,
    group_id: &Uuid,
    user_id: &Uuid,
    req: NotesGroupPatchRequest,
) -> Result<SmalltalkNotesGroup, sqlx::Error> {
    let mut rb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE smalltalk_notes_groups SET ");
    let mut first = true;

    macro_rules! set {
        ($col:literal, $value:expr) => {{
            if !first {
                rb.push(", ");
            }
            first = false;
            rb.push(concat!($col, " = ")).push_bind($value);
        }};
    }

    if let Some(name) = req.enc_group_name {
        set!("enc_group_name", name);
    }

    if let Some(group_metadata) = req.enc_group_metadata {
        set!("enc_group_metadata", group_metadata);
    } else if req.is_deleted == Some(true) {
        if !first {
            rb.push(", ");
        }
        first = false;
        rb.push("enc_group_metadata = NULL");
    }

    if let Some(rank) = req.rank {
        set!("rank", rank);
    }

    if let Some(deleted) = req.is_deleted {
        set!("is_deleted", deleted);
    }

    if !first {
        rb.push(", ");
    }

    rb.push("updated_at = NOW()");

    rb.push(" WHERE id = ")
        .push_bind(group_id)
        .push(" AND user_id = ")
        .push_bind(user_id)
        .push(" RETURNING *");

    let query = rb.build_query_as::<SmalltalkNotesGroup>();
    let updated_group = query.fetch_one(pool).await?;

    Ok(updated_group)
}
