const puppeteer = require('puppeteer');

function delay(time) {
    return new Promise(function(resolve) {
        setTimeout(resolve, time)
    });
}

(async () => {
    // Launch the browser and open a new blank page
    var a = performance.now();
    const browser = await puppeteer.launch({headless: "new"});
    var b = performance.now();
    console.log(b - a);
    const page = await browser.newPage();
    var c = performance.now();
    console.log(c - b);
    await page.setViewport({width: 1920, height: 1080});
    var d = performance.now();
    console.log(d - c);

    // Navigate the page to a URL
    await page.goto('http://127.0.0.1:8080/test.html');
    var e = performance.now();
    console.log(e - d);

    await page.waitForSelector("#done");
    var f = performance.now();
    console.log(f - e);

    await page.screenshot({fullPage: true, path: "out.png"});
    var g = performance.now();
    console.log(g - f);

    await browser.close();
    var h = performance.now();
    console.log(h - g);
})();