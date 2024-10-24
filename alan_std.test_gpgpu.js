import assert from "node:assert";
import { chromium } from 'playwright';

// Currently can only run this on Windows and MacOS. Chromium has WebGPU behind flags on Linux that
// Playwright doesn't support setting
(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('http://localhost:8080/alan_std.test.html');

  assert.strictEqual(await page.evaluate(async () => {
    let error = "";
    try {
      await alanStd.gpu();
    } catch (e) {
      error = e.message;
    }
    return error;
  }), "");

  await context.close();
  await browser.close();
})();
