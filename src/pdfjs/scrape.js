const puppeteer = require('puppeteer');

function delay(time) {
    return new Promise(function(resolve) {
        setTimeout(resolve, time)
    });
}

(async () => {
    // Launch the browser and open a new blank page
    const browser = await puppeteer.launch({headless: 'new'});
    const page = await browser.newPage();

    // Navigate the page to a URL
    await page.goto('http://127.0.0.1:5500/test.html');

    await page.waitForSelector("#done");
    const element = await page.waitForSelector('img');
    console.log(element);

    await browser.close();
})();