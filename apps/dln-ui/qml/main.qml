import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import net.devinlittle.dln_ui

Window {
    visible: true
    color: "#ff0000"
    width: 400
    height: 300
    title: "DLN Counter " + myObject.counter

    MyObject {
        id: myObject
    }

    ColumnLayout {
        anchors.verticalCenterOffset: 0
        anchors.horizontalCenterOffset: 7
        anchors.centerIn: parent
        spacing: 20

        Rectangle {
          color: "lightgray"

          Text {
              id: counterText
              text: "hi"
              font.pixelSize: 72
              Layout.alignment: Qt.AlignHCenter
              Layout.fillWidth: true
              horizontalAlignment: Text.AlignHCenter
            }
          }

        RowLayout {
            spacing: 16
            Layout.alignment: Qt.AlignHCenter

            Button {
                text: "-"
                font.pixelSize: 48
                width: 80
                height: 80
                onClicked: myObject.decrement()
            }

            Button {
                text: "+"
                font.pixelSize: 48
                width: 80
                height: 80
                onClicked: myObject.increment()
            }
        }

        Button {
            text: "Reset"
            Layout.alignment: Qt.AlignHCenter
            onClicked: myObject.reset()
        }
    }
}