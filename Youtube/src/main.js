const amqp = require('amqplib');
const { EventEmitter } = require('events');
const scraperController = require('../lib/pageController');


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
let channelId = getEnvVar("YT_CHANNEL_ID", "");
let liveCheckSleep = getEnvVar("CHECK_FOR_LIVE_INTERVAL", 5 * 60 * 1000); // Default to five minutes in millis

let emitter = new EventEmitter;
scraperController(channelId, emitter, liveCheckSleep);

const key = 'youtube';

let url = "amqp://" + username + ":" + password + "@" + ampqHost + ":" + ampqPort; 

amqp.connect(url).then(function(conn) {
    return conn.createChannel().then(function(ch) {
        const ex = exchange;
        const ok = ch.assertExchange(ex, 'topic', {durable: true});
        return ok.then(function() {
            console.log("Connected to RabbitMQ");
            emitter.on('message', (data) => {
                var message = {
                    from: "Youtube",
                    source_badge_large: "https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon_32x32.png",
                    source_badge_small: "https://www.youtube.com/s/desktop/f9ccd8c6/img/favicon.ico",
                    message: data.message,
                    raw_message: data.raw_message,
                    username: data.username,
                    user_color_r: "FF",
                    user_color_g: "00",
                    user_color_b: "00",
                    user_badges: [
                        ""
                    ],
                    message_emotes: data.emotes
                }

                console.log(data);

                ch.publish(ex, key, Buffer.from(JSON.stringify(message)));
            });
        });
    });
});