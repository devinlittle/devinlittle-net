use dln_core::add;

use core::pin::Pin;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QUrl};
use cxx_qt_lib_extras::QApplication;
use std::env;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, counter)]
        #[namespace = "dln"]
        type MyObject = super::MyObjectRust;

        #[qinvokable]
        fn increment(self: Pin<&mut Self>);

        #[qinvokable]
        fn decrement(self: Pin<&mut Self>);

        #[qinvokable]
        fn reset(self: Pin<&mut Self>);
    }
}

#[derive(Default)]
pub struct MyObjectRust {
    counter: i32,
}

impl qobject::MyObject {
    pub fn increment(self: Pin<&mut Self>) {
        let new_value = *self.counter() + 1;
        println!("increased to {}", new_value);
        self.set_counter(new_value);
    }

    pub fn decrement(self: Pin<&mut Self>) {
        let new_value = *self.counter() - 1;
        println!("decreased to {}", new_value);
        self.set_counter(new_value);
    }

    pub fn reset(self: Pin<&mut Self>) {
        self.set_counter(0);
        println!("reset the counter");
    }
}

fn main() {
    let mut app = QApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    QGuiApplication::set_desktop_file_name(&"net.devinlittle.dln".into());

    if env::var("QT_QUICK_CONTROLS_STYLE").is_err() {
        QQuickStyle::set_style(&"Basic".into());
    }

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(
            "qrc:/qt/qml/net/devinlittle/dln_ui/qml/main.qml",
        ));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
