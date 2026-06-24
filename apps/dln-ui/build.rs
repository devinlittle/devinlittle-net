use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("net.devinlittle.dln_ui").qml_files(["qml/main.qml", "qml/hello.qml"]),
    )
    .files(["src/main.rs"])
    .build();
}
