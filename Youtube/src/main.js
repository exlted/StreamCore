const amqp = require('amqplib');
const YoutubeChat = require("../lib/client");

function getEnvVar(variable, defaultVal) {
    if (variable in process.env) {
        return process.env[variable];
    } else {
        return defaultVal;
    }
}


let ampqHost = getEnvVar("AMPQ_HOST", "localhost");
let ampqPort = getEnvVar("AMPQ_PORT", "5672");
let username = getEnvVar("AMPQ_USERNAME", "guest");
let password = getEnvVar("AMPQ_PASSWORD", "guest");
let exchange = getEnvVar("EXCHANGE_NAME", "chat");
let channelId = getEnvVar("YT_CHANNEL_ID", "UCgHUiD9lbIgi1y8pMBUuiNQ");



const yt = new YoutubeChat({channelId: channelId});

const key = 'youtube';

function tryConnect() {
    let findNewLive = setInterval(() => {
        if(yt.connect().await) {
            clearInterval(findNewLive);
        }
    }, 60000);
    yt.connect();
}

tryConnect();


yt.on('start', ()=> {
    console.log('Connected to YouTube!');
});

yt.on('error', error => {
    console.log(error);
});

let url = "amqp://" + username + ":" + password + "@" + ampqHost + ":" + ampqPort; 

amqp.connect(url).then(function(conn) {
    return conn.createChannel().then(function(ch) {
        const ex = exchange;
        const ok = ch.assertExchange(ex, 'topic', {durable: true});
        return ok.then(function() {
            console.log("Connected to RabbitMQ");
            yt.on('message', (data) => {
                var message = {
                    from: "Youtube",
                    source_badge_large: "https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon_32x32.png",
                    source_badge_small: "https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon.ico",
                    message: data.message.runs[0].text,
                    raw_message: data.message.runs[0].text,
                    username: data.authorName.simpleText,
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

yt.on('live_ended', () => {
    yt.stop();
    tryConnect();
});