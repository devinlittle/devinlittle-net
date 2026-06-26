import QtQuick

Item {
    id: root
    width: 110
    height: 80

    property int value: 0
    property string textColor: "#9ac7ff"

    property int displayValue: value
    onValueChanged: {
        if (value > displayValue) {
            outText.text = displayValue;
            inText.text = value;

            outText.y = 0;
            inText.y = root.height;

            slideUpAnim.restart();
        } else if (value < displayValue) {
            outText.text = displayValue;
            inText.text = value;

            outText.y = 0;
            inText.y = -root.height;

            slideDownAnim.restart();
        }
        displayValue = value;
    }

    clip: true

    Text {
        id: outText
        width: root.width
        height: root.height
        font.pixelSize: 72
        font.family: "Inter"
        color: root.textColor
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        text: root.value.toString()
    }

    Text {
        id: inText
        width: root.width
        height: root.height
        font.pixelSize: 72
        font.family: "Inter"
        color: root.textColor
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        y: root.height
    }

    ParallelAnimation {
        id: slideUpAnim
        NumberAnimation {
            target: outText
            property: "y"
            to: -root.height
            duration: 250
            easing.type: Easing.OutCubic
        }
        NumberAnimation {
            target: inText
            property: "y"
            to: 0
            duration: 250
            easing.type: Easing.OutCubic
        }
    }

    ParallelAnimation {
        id: slideDownAnim
        NumberAnimation {
            target: outText
            property: "y"
            to: root.height
            duration: 250
            easing.type: Easing.OutCubic
        }
        NumberAnimation {
            target: inText
            property: "y"
            to: 0
            duration: 250
            easing.type: Easing.OutCubic
        }
    }
}
