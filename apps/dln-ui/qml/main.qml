import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import net.devinlittle.dln_ui

Window {
    visible: true
    color: "#1d1130"
    width: 400
    height: 300
    title: "DLN Counter: " + myObject.counter

    MyObject {
        id: myObject
    }

    Item {
        anchors.fill: parent
        anchors.margins: 16
        anchors.leftMargin: 0
        anchors.rightMargin: 32
        anchors.topMargin: 0
        anchors.bottomMargin: 32

        AnimatedCounter {
            id: counterText
            x: 145
            y: 94
            width: 110
            height: 80
            value: myObject.counter
            textColor: "#9ac7ff"
        }

        Button {
            id: decButton
            x: 50
            y: 130
            width: 80
            height: 80
            text: "-"

            contentItem: Text {
                text: decButton.text
                color: "#eae6f2"
                font.pixelSize: 48
                font.family: "Inter"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            background: Rectangle {
                color: decButton.down ? "rgba(255, 255, 255, 0.15)" : "rgba(255, 255, 255, 0.05)"
                border.color: Qt.rgba(1.0, 1.0, 1.0, 0.1)
                border.width: 1
                radius: 8
            }

            onClicked: myObject.decrement()
        }

        Button {
            id: incButton
            x: 270
            y: 130
            width: 80
            height: 80
            text: "+"

            contentItem: Text {
                color: "#eae6f2"
                text: "+"
                font.pixelSize: 48
                font.family: "Inter"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            background: Rectangle {
                color: incButton.down ? "rgba(255, 255, 255, 0.15)" : "rgba(255, 255, 255, 0.05)"
                border.color: Qt.rgba(1.0, 1.0, 1.0, 0.1)
                border.width: 1
                radius: 8
            }

            onClicked: myObject.increment()
        }

        Button {
            id: resetButton
            x: 150
            y: 230
            width: 100
            height: 40
            text: "Reset"

            contentItem: Text {
                text: resetButton.text
                color: "#7e9cff"
                font.pixelSize: 16
                font.family: "Inter"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            background: Rectangle {
                color: resetButton.down ? "rgba(255, 255, 255, 0.15)" : "rgba(255, 255, 255, 0.05)"
                border.color: Qt.rgba(1.0, 1.0, 1.0, 0.1)
                border.width: 1
                radius: 6
            }

            onClicked: myObject.reset()
        }
    }
}
