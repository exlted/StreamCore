const Utils = {
  chatIsOffline(message) {
    let regex = /"contents":{"messageRenderer":{"text":/i
    return !!message.match(regex);
  },
  usecToTime: usec => Math.floor(Number(usec) / 1000)
}

module.exports = Utils;