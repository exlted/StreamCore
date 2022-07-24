const browserObject = require('./browser');
const scrapers = require('./pageScraper');

const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds))
}

async function scrapeAll(channelID, emitter, liveCheckSleep){
    let browserInstance = browserObject.startBrowser();
    let browser;
    try{
        browser = await browserInstance;
        let ID = "";
        while(true) {
            let tempID = await scrapers.hasLiveScraper.scraper(browser, channelID);
            if (tempID != ID) {
                emitter.emit('live_ended');
                if (tempID != "") {
                    scrapers.chatScraper.scraper(browser, tempID, emitter);
                }
                ID = tempID;
            }   
            await sleep(liveCheckSleep); // Configurable sleep timer, defaults to 5 minutes
        }
    }
    catch(err){
        console.log("Could not resolve the browser instance => ", err);
    }
}

module.exports = (channelID, emitter, liveCheckSleep) => scrapeAll(channelID, emitter, liveCheckSleep)
