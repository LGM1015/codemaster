const { chromium } = require('playwright');

(async () => {
  const url = process.argv[2];
  if (!url) {
    console.error('Usage: node inspect.js <url>');
    process.exit(1);
  }

  const browser = await chromium.launch();
  const page = await browser.newPage();
  const logs = [];

  page.on('console', msg => logs.push(`[${msg.type()}] ${msg.text()}`));
  page.on('pageerror', err => logs.push(`[error] ${err.toString()}`));

  try {
    await page.goto(url, { waitUntil: 'networkidle' });
    const title = await page.title();
    const text = await page.evaluate(() => document.body.innerText);
    
    console.log(JSON.stringify({
      title,
      logs,
      text: text.substring(0, 10000)
    }, null, 2));
  } catch (e) {
    console.error(JSON.stringify({ error: e.message }));
  } finally {
    await browser.close();
  }
})();
