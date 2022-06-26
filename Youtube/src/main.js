const amqp = require('amqplib');
const YoutubeChat = require("../lib/client");
const { exit } = require('process');

const yt = new YoutubeChat({channelId: "UCgHUiD9lbIgi1y8pMBUuiNQ"});

const key = 'youtube';
yt.connect();

yt.on('start', ()=> {
    console.log('Connected to YouTube!');
})

yt.on('error', error => {
    console.error(error);
    exit;
})

amqp.connect("amqp://guest:guest@localhost:5672").then(function(conn) {
    return conn.createChannel().then(function(ch) {
        const ex = 'chat';
        const ok = ch.assertExchange(ex, 'topic', {durable: true});
        return ok.then(function() {
            yt.on('message', (data) => {
                var message = {
                    from: "Youtube",
                    source_badge_large: "view-source:https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon_32x32.png",
                    source_badge_small: "view-source:https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon.ico",
                    message: data.message.runs[0].text,
                    raw_message: data.message.runs[0].text,
                    username: data.authorName,
                    user_color_r: "FF",
                    user_color_g: "FF",
                    user_color_b: "FF",
                    user_badges: [
                        ""
                    ]
                }
                
                ch.publish(ex, key, Buffer.from(JSON.stringify(message)));
            });
        });
    });
});