use core::pin::Pin;
use cxx_qt::Threading;
use dln_core::{
    auth::AuthError::*,
    error::CoreError::{self},
    helpers::auth::get_username,
};

use crate::loading::qobject::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, is_loaded)]
        #[qproperty(QString, loading_text)]
        #[qproperty(QString, username)]
        #[namespace = "dln"]
        type AppState = super::MyAppState;

        #[qinvokable]
        fn start_loading(self: Pin<&mut Self>);

        #[qsignal]
        fn loading_finished(self: Pin<&mut Self>);

        #[qsignal]
        fn loading_failed(self: Pin<&mut Self>, error: QString);

    }
    impl cxx_qt::Threading for AppState {}
}

#[derive(Default)]
pub struct MyAppState {
    is_loaded: bool,
    username: QString,
    loading_text: QString,
}

impl qobject::AppState {
    pub fn start_loading(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        tokio::spawn(async move {
            let result = dln_core::init().await;

            qt_thread
                .queue(move |mut app_state| match result {
                    Ok(_) => {
                        app_state.as_mut().set_is_loaded(true);
                        app_state
                            .as_mut()
                            .set_username(get_username().unwrap().into());
                    }
                    Err(CoreError::Auth(auth_err)) => match auth_err {
                        Unauthenticated | Unauthorized => {
                            app_state.loading_failed("Check your username or password.".into());
                        }
                        InternalServerError => {
                            app_state.loading_failed("The server had an issue.".into());
                        }
                        AccountLocked => {
                            app_state.loading_failed("Account suspended.".into());
                        }
                        RequestFailure => {
                            app_state.loading_failed("Connection failed.".into());
                        }
                    },
                    _ => {
                        app_state.loading_failed("App failure".into());
                    }
                })
                .unwrap();
        });
    }
}
