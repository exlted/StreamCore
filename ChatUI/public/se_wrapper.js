// This is a wrapper to wrap StreamCore events and translate them into a format that StreamElements chat customizations will understand
document.addEventListener("messageRecieved", function(e) {
    let data = {
        text: e.detail.raw_message,
        emotes: [],
        nick: e.detail.username,
        badges: [],
        displayColor: "#" + e.detail.user_color_r + e.detail.user_color_g + e.detail.user_color_b,
        displayName: e.detail.username,
        isAction: false
    };

    e.detail.message_emotes.forEach(element => {
        data.emotes.push(element);
    });

    e.detail.user_badges.forEach(element => {
        data.badges.push({
            url: element
        })
    });

    if (window.global.streamelements.sourceAsBadge) {
        let url = e.detail.source_badge_small
        if (window.global.streamelements.useLargeSource) {
            url = e.detail.source_badge_large;
        }
        data.badges.push({
            url: url
        });
    }

    let seEvent = new CustomEvent("onEventReceived", {
        detail: {
            listener: "message",
            event: {
                data: data
            }
        }
    });

    window.dispatchEvent(seEvent);
})
