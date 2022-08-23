const { fail } = require("assert");

const blank = "about:blank";

async function getFirstEmptyPage(browser) {
    let pages = await browser.pages()
    for (const potentialPage of pages) {
        if (await potentialPage.url() == blank) {
            // Use an existing page
            return potentialPage;
        }
    }
    // Make a new page
    return await browser.newPage();
}


const chatScraper = {
	url: 'https://www.youtube.com/live_chat?is_popout=1&v=',
    async scraper(browser,videoID,emitter){
        const page = await getFirstEmptyPage(browser);
        let url = this.url + videoID;
        console.log(`Navigating to ${url}...`);
        await page.goto(url);
        // Looking at this, I think we want to sit in a loop, wait on DOM updates

        await page.exposeFunction('puppeteerPageMutated', item => {
            emitter.emit("message", item);
        });

        await page.evaluate(() => {
            const target = document.querySelector('#item-offset');
            const observer = new MutationObserver( mutations => {
                for (const mutation of mutations) {
                    if (mutation.type === 'childList') {
                        if (mutation.addedNodes.length != 0) {
                            mutation.addedNodes.forEach(element => {
                                if (element.tagName == "YT-LIVE-CHAT-TEXT-MESSAGE-RENDERER") {
                                    let emotes = [];
                                    let message = element.querySelector("#message");
                                    for (child of message.children) {
                                        if (child.tagName == "IMG") {
                                            let name = "";
                                            let url = "";

                                            for (attr of child.attributes) {
                                                if (attr.name == "shared-tooltip-text") {
                                                    name = attr.value;
                                                    continue;
                                                }
                                                if (attr.name == "src") {
                                                    url = attr.value;
                                                    continue;
                                                }
                                            }

                                            emotes.push({
                                                url: url,
                                                name: name
                                            });
                                        }
                                    }
                                    let rawHTML = message.innerHTML;
                                    let cleaned_msg = rawHTML.replace(/<.*?shared-tooltip-text="(.*?)".*?>/gm, " $1 ");
                                    cleaned_msg = cleaned_msg.replace(/<.*?>/gm, "");

                                    let data = {
                                        message: rawHTML,
                                        raw_message: cleaned_msg,
                                        username: element.querySelector("#author-name").innerText,
                                        emotes: emotes
                                    }
                                    console.log(data);
                                    puppeteerPageMutated(data);
                                }
                            });
                        }
                    }
                }
            });
            observer.observe(target, {childList: true, subtree: true});
        });

        emitter.on("live_ended", () => {
            console.log("Live Ended");
            page.close();
        });
	}
}

const hasLiveScraper = {
    liveChatURL: `https://www.youtube.com/channel/`,
    async scraper(browser, channelID) {
        const page = await getFirstEmptyPage(browser);
        let url = this.liveChatURL + channelID + "/live";
        console.log(`Navigating to ${url}...`);
        await page.goto(url);
        await page.waitForTimeout(1000);
        let newURL = await page.url();
        let rv = "";
        if (newURL != url) {
            let videoID = await page.evaluate(() => {
                let params = new URLSearchParams(document.location.search);
                return params.get("v");
            });
            rv = videoID;
        }
        page.goto(blank);
        return rv;
    }
}

module.exports = {chatScraper, hasLiveScraper};
