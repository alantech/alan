import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({
      args: [
        '--enable-unsafe-webgpu',
        '--disable-features=OutOfBlinkCors',
        '--disable-gpu-sandbox',
        '--ignore-gpu-blocklist',
      ],
      headless: true,
  });
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on('console', msg => process.stdout.write(msg.text() + '\n'));
  page.on('pageerror', err => console.error(err.message));
  await page.goto(process.argv.pop());
  await new Promise((r) => setTimeout(r, 15000)); // Increased from 5s to 15s for GPU init
  await context.close();
  await browser.close();
})();
