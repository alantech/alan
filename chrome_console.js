import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on('console', msg => console.log(msg.text()));
  await page.goto(process.argv.pop());
  await new Promise((r) => setTimeout(r, 2000)); // TODO: Better way to determine completion
  await context.close();
  await browser.close();
})();
