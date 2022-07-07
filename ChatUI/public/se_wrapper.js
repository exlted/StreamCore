// This is a wrapper to wrap StreamCore events and translate them into a format that StreamElements chat customizations will understand
document.addEventListener("messageRecieved", function(e) {
    let data = {
        text: e.detail.message,
        emotes: [],
        nick: e.detail.username,
        badges: [],
        displayColor: e.detail.user_color_r + e.detail.user_color_g + e.detail.user_color_b,
        displayName: e.detail.username,
        isAction: false
    };

    e.detail.user_badges.forEach(element => {
        data.badges.push({
            url: element
        })
    });

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
