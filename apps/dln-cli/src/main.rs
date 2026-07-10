use dln_core::{auth::AuthError, error::CoreError, structs::LoginPayload};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    println!("Initializing DLN Session...");

    if let Err(err) = dln_core::init().await {
        match err {
            CoreError::Auth(AuthError::Unauthorized)
            | CoreError::Auth(AuthError::Unauthenticated) => {
                println!("\nPlease log in.");

                if let Err(login_err) = handle_interactive_login().await {
                    eprintln!("Login failed: {:?}", login_err);
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Critical system error during startup: {:?}", err);
                std::process::exit(1);
            }
        }
    }

    println!("Hello from the CLI!!");
}

async fn handle_interactive_login() -> Result<(), CoreError> {
    let mut username = String::new();

    print!("Username: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut username)
        .map_err(|_| CoreError::Auth(AuthError::InternalServerError))?;
    let username = username.trim().to_string();

    let password = rpassword::prompt_password("Password: ")
        .map_err(|_| CoreError::Auth(AuthError::InternalServerError))?;

    println!("Authenticating...");

    let payload = LoginPayload { username, password };
    dln_core::auth::login_req(payload).await?;

    println!("Successfully authenticated");
    Ok(())
}
