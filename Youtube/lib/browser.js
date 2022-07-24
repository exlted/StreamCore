const puppeteer = require('puppeteer-extra');

const StealthPlugin = require('puppeteer-extra-plugin-stealth');
puppeteer.use(StealthPlugin());

async function startBrowser(headless){
	let browser;
	try {
	    console.log("Opening the browser......");
	    browser = await puppeteer.launch({
	        headless: true,
	        args: ["--disable-setuid-sandbox"],
	        'ignoreHTTPSErrors': true
	    });
	} catch (err) {
	    console.log("Could not create a browser instance => : ", err);
	}
	return browser;
}

module.exports = {
	startBrowser
};