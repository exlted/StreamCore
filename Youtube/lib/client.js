const { EventEmitter } = require('events');
const axios = require('axios');

class YoutubeChat extends EventEmitter {
  constructor({ channelId }) {
    super();
    this.channelId = channelId;
    this.headers = {
      'User-Agent': 'Mozilla/5.0 (Mac) Gecko/20100101 Firefox/76.0',
      'x-youtube-client-name': '1',
      'x-youtube-client-version': '2.20200603.01.00',
    };
    this.liveURL = `https://www.youtube.com/channel/${channelId}/live`;
    this.interval = 500;
    this.prevTime = 0;
    this.observer = null;
  }

  stop(message) {
    if (this.observer) {
      clearInterval(this.observer);
      this.emit('end', message);
    }
  }

  async connect() {
    const liveResponse = await axios.get(this.liveURL, { headers: this.headers });

    const getNonce = /<script nonce=\".*?\">(.*?)<\/script>/gi;
    let test = liveResponse.data.split(getNonce);
    let data = "";
    for (let i = 0; i < test.length; ++i) {
      let string = test[i];
      if (string.startsWith("var ytInitialPlayerResponse =")) {
        let initialIndex = string.indexOf("{");
        data = string.substr(initialIndex, string.length - initialIndex - 1);
        break;
      }
    }

    if (data == "") {
      this.emit('error', new Error("No Livestream Found"));
      return false;
    }

    data = data.substring(0, data.lastIndexOf('}') + 1);
    let pageData = JSON.parse(data);
    let videoInfo = pageData.videoDetails;

    this.liveId = videoInfo.videoId;

    if (!this.liveId) {
      this.emit('error', new Error('Live stream not found'));
      return false;
    }

    this.observer = setInterval(() => this.getChatMessages(), this.interval);

    this.emit('start', this.liveId);

    return true;
  }
  async getChatMessages() {
    try {
      const liveChatURL = `https://www.youtube.com/live_chat?v=${this.liveId}`;
      const response = await axios.get(liveChatURL, { headers: this.headers });

      const getNonce = /<script nonce=\".*?\">(.*?)<\/script>/gi;
      let test = response.data.split(getNonce);
      let data = "";
      for (let i = 0; i < test.length; ++i) {
        let string = test[i];
        if (string.startsWith("window")) {
          let initialIndex = string.indexOf("{");
          data = string.substr(initialIndex, string.length - initialIndex - 1);
          break;
        }
      }

      let pageData = JSON.parse(data);

      if (pageData.contents.liveChatRenderer) {
        pageData.contents.liveChatRenderer.actions.forEach(item => {
          let data = item.addChatItemAction.item.liveChatTextMessageRenderer;
          if (data) {
            if (data.timestampUsec > this.prevTime) {
              this.emit('message', data);
              this.prevTime = data.timestampUsec;
            }
          }
        });
      } else {  
        this.emit('live_ended');
        return console.log('Live stream offline');
      }
    } catch (err) {
      console.log(err);
    }
  }
}

module.exports = YoutubeChat