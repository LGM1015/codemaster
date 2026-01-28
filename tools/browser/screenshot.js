const { chromium } = require('playwright');
const path = require('path');

(async () => {
  const url = process.argv[2];
  const outputPath = process.argv[3] || 'screenshot.png';
  
  if (!url) {
    console.error('Usage: node screenshot.js <url> [output_path]');
    process.exit(1);
  }

  const browser = await chromium.launch();
  const page = await browser.newPage();

  try {
    await page.goto(url, { waitUntil: 'networkidle' });
    await page.screenshot({ path: outputPath, fullPage: true });
    console.log(JSON.stringify({ success: true, path: path.resolve(outputPath) }));
  } catch (e) {
    console.error(JSON.stringify({ error: e.message }));
  } finally {
    await browser.close();
  }
})();
