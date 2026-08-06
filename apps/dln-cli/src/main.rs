use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use dln_core::{
    auth::{AuthError, logout},
    error::CoreError,
    event_bus::EventBusEvent,
    helpers::gradegetter::grab_grades,
    services::gradegetter::GradeOutput,
    structs::{ForwardStatus, LoginPayload, SchoologyLogin},
};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Password, PasswordDisplayMode, Text};
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, value_enum)]
    pub generate_completion: Option<Shell>,
}

#[derive(Subcommand)]
pub enum Commands {
    // Auth
    /// Login to DLN
    Login {
        #[arg(long, short = 'u', env = "DLN_USERNAME")]
        username: Option<String>,

        #[arg(long, short = 'p', env = "DLN_PASSWORD")]
        password: Option<String>,
    },
    /// Log Out of DLN
    Logout,

    /// Functions related to gradegetter [supports gg, gradegetter, and grade_getter]
    #[command(alias = "gg", alias = "grade_getter")]
    Gradegetter {
        #[command(subcommand)]
        action: GradeAction,
    },
}

#[derive(Subcommand)]
pub enum GradeAction {
    /// fetch and display your grades
    #[command(alias = "fetch")]
    Get {
        #[arg(long, short = 'j')]
        json: bool,
    },

    /// link your schoology with gradegetter
    LoginSchoology {
        #[arg(long, short = 'u')]
        schoology_email: Option<String>,

        #[arg(long, short = 'p')]
        schoology_password: Option<String>,
    },

    /// Delete stored schoology credentials
    DeleteCreds,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Login { username, password }) = cli.command {
        if dln_core::init().await.is_ok() {
            println!("Already Logged In");
            std::process::exit(1);
        }
        let _ = handle_login(username, password).await;
        std::process::exit(1);
    }

    if let Some(Commands::Logout) = cli.command {
        let _ = dln_core::init().await;
        if logout().await.is_ok() {
            println!("Logged out Sucessfully.");
        }
        std::process::exit(1);
    }

    // only init when a command is given
    if cli.command.is_some() {
        if let Err(err) = dln_core::init().await {
            match err {
                CoreError::Auth(AuthError::Unauthorized)
                | CoreError::Auth(AuthError::Unauthenticated) => match cli.command {
                    Some(Commands::Logout) => {
                        return logout().await.map_err(|e| e.into());
                    }
                    _ => {
                        println!("Please log in first");
                        std::process::exit(1);
                    }
                },
                _ => {
                    eprintln!("Critical system error during startup: {:?}", err);
                    std::process::exit(1);
                }
            }
        }

        {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<EventBusEvent>(256);

            dln_core::event_bus::init_events(tx);
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    println!("{event:?}");

                    match event {
                        EventBusEvent::GradesUpdated => {
                            if let Ok(grades) = grab_grades() {
                                println!("{grades:?}");
                            }
                        }
                    }
                }
            });
        }
    }

    match cli.command {
        Some(Commands::Gradegetter { action }) => match action {
            GradeAction::Get { json } => {
                if !json {
                    let grades = dln_core::services::gradegetter::fetch_grades(false).await?;

                    let grades = match grades {
                        GradeOutput::BTreeGrades(grades) => Ok(grades),
                        GradeOutput::JsonGrades(_) => {
                            Err("Expected BTreeGrades, got JsonGrades".to_string())
                        }
                    }
                    .map_err(|_| CoreError::RequestFailure)?;

                    for (subject, scores) in grades {
                        let mut table = Table::new();
                        table.set_content_arrangement(ContentArrangement::Dynamic);
                        table.set_width(85);

                        table.set_header(vec![
                            Cell::new(subject).fg(Color::Cyan),
                            Cell::new("Quarter"),
                        ]);

                        for (i, score) in scores.iter().enumerate() {
                            let quarter = format!("Q{}", i + 1);

                            let score_cell = match score {
                                Some(value) => {
                                    let color = if *value >= 90.0 {
                                        Color::Green
                                    } else if *value >= 75.0 {
                                        Color::Yellow
                                    } else {
                                        Color::Red
                                    };

                                    Cell::new(format!("{:.2}", value)).fg(color)
                                }
                                None => Cell::new("N/A").fg(Color::Grey),
                            };

                            let mut row = Row::new();
                            row.add_cell(Cell::new(quarter));
                            row.add_cell(score_cell);
                            table.add_row(row);
                        }

                        println!("{}", table);
                    }
                } else {
                    let grades = dln_core::services::gradegetter::fetch_grades(true).await?;

                    let grades = match grades {
                        GradeOutput::BTreeGrades(_) => {
                            Err("Expected JsonGrades, got BTreeGrades".to_string())
                        }
                        GradeOutput::JsonGrades(grades) => Ok(grades),
                    }
                    .map_err(|_| CoreError::RequestFailure)?;

                    println!("{grades}");
                }
            }
            GradeAction::LoginSchoology {
                schoology_email,
                schoology_password,
            } => {
                let schoology_email = schoology_email.unwrap_or_else(|| {
                    Text::new("Schoology Email:")
                        .with_help_message("Enter your email ending in @hawks.tech")
                        .prompt()
                        .expect("Failed to read username")
                });

                let schoology_password = schoology_password.unwrap_or_else(|| {
                    Password::new("Schoology Password:")
                        .with_display_mode(PasswordDisplayMode::Masked)
                        .prompt()
                        .expect("Failed to read password")
                });

                let schoology_login = SchoologyLogin {
                    schoology_email,
                    schoology_password,
                };

                let pb = ProgressBar::new(7);
                pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) - {msg}")
                    .unwrap()
                    .progress_chars("█▓░"),
                );

                dln_core::helpers::gradegetter::add_schoology_credentials(schoology_login).await?;
                let mut rx = dln_core::services::gradegetter::forward_ws().await?;

                while rx.changed().await.is_ok() {
                    let status = rx.borrow().clone();
                    if status == ForwardStatus::ErrorInSetup {
                        println!("Error During Setup");
                        println!("Email/Password is probably incorrect, check what you typed in.");
                        break;
                    }

                    let text_to_print = match status {
                        ForwardStatus::Started => "Started Process!",
                        ForwardStatus::Navigated => "Navigated to Schoology's page.",
                        ForwardStatus::TypedEmail => "Typed in Email",
                        ForwardStatus::EnteredEmail => "Entered in Email",
                        ForwardStatus::TypedPassword => "Typed in Password",
                        ForwardStatus::EnteredPassword => "Entered in Password",
                        ForwardStatus::Finished => "Finished!, you are setup for gradegetter",
                        _ => "",
                    };

                    println!();
                    println!("{}", text_to_print.bold());
                    pb.inc(1);
                }
            }
            GradeAction::DeleteCreds => {
                match dln_core::helpers::gradegetter::delete_credentials()
                    .await
                    .is_ok()
                {
                    true => {
                        println!("Sucessfully deleted credentials");
                    }
                    false => {
                        println!("Failed to deleted credentials");
                    }
                }
            }
        },
        _ => {
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    };

    Ok(())
}

async fn handle_login(username: Option<String>, password: Option<String>) -> Result<(), CoreError> {
    let username = username.unwrap_or_else(|| {
        Text::new("Username:")
            .with_help_message("Enter your username 🥶:")
            .prompt()
            .expect("Failed to read username")
    });

    let password = password.unwrap_or_else(|| {
        Password::new("Password:")
            .with_display_mode(PasswordDisplayMode::Masked)
            .prompt()
            .expect("Failed to read password")
    });

    dln_core::auth::login_req(LoginPayload { username, password }).await
}
