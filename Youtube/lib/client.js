const { EventEmitter } = require('events');
const { inherits } = require('util');
const axios = require('axios');
const Utils = require('./utils')


function YoutubeChat({ channelId }) {
  this.channelId = channelId;
  this.headers = {
    'User-Agent': 'Mozilla/5.0 (Mac) Gecko/20100101 Firefox/76.0',
    'x-youtube-client-name': '1',
    'x-youtube-client-version': '2.20200603.01.00',
  }
  this.liveURL = `https://www.youtube.com/channel/${channelId}/live`
  this.interval = 100
  this.prevTime = 0
}

YoutubeChat.prototype.connect = async function() {
  const liveResponse = await axios.get(this.liveURL, {headers: this.headers})
  
  //if (liveResponse.data.match(/LIVE_STREAM_OFFLINE/)) {
  //  this.emit('error', new Error("Live stream offline"))
  //  return false
  //}
  const liveId = liveResponse.data.match(/"watchEndpoint":{"videoId":"(\S*?)"}/)
  this.liveId = liveId ? liveId[1] : "";

  if (!this.liveId) {
    this.emit('error', new Error('Live stream not found'))
    return false
  }

  if (this.liveId.includes('"')) {
    this.liveId = this.liveId.substr(0, this.liveId.indexOf('"'));
  }

  this.observer = setInterval(() => this.getChatMessages(), this.interval)

  this.emit('start', this.liveId)

  return true;
}

YoutubeChat.prototype.stop = message => {
  if (this.observer) {
    clearInterval(this.observer)
    this.emit('end', message)
  }
}

YoutubeChat.prototype.getChatMessages = async function() {
  const liveChatURL = `https://www.youtube.com/live_chat?v=${this.liveId}`
  const response = await axios.get(liveChatURL, {headers: this.headers})

  if(Utils.chatIsOffline(response.data)) {
    return console.log('Live stream offline')
  }
  
  const getNonce = /<script nonce=\".*?\">(.*?)<\/script>/gi
  let test = response.data.split(getNonce);
  let data = "";
  for (let i = 0; i < test.length; ++i) {
    string = test[i];
    if (string.startsWith("window")) {
      let initialIndex = string.indexOf("{");
      data = string.substr(initialIndex, string.length - initialIndex - 1);
      break;
    }
  }

  try {
    let pageData = JSON.parse(data);
  
    pageData.contents.liveChatRenderer.actions.forEach(item => {
      let data = item.addChatItemAction.item.liveChatTextMessageRenderer;
      if (data) {
        if (Utils.usecToTime(data.timestampUsec) > this.prevTime) {
          this.emit('message', data)
          this.prevTime = Utils.usecToTime(data.timestampUsec);
        }
      }
    });
  } catch(err) {
    console.log(err);
  }
}

inherits(YoutubeChat, EventEmitter);

module.exports = YoutubeChat