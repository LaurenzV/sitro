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
    await page.goto('http://127.0.0.1:8080/test.html');

    await page.waitForSelector("#done");

    let content = await page.evaluate(() => {
        let divs = [...document.querySelectorAll('div')];
        return divs.map((div) => div.id);
    });

    console.log(content);

    await browser.close();
})();