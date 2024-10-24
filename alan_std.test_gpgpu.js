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

  assert.strictEqual(await page.evaluate(async () => {
    let b = await alanStd.createEmptyBuffer(alanStd.storageBufferType(), 4);
    return alanStd.bufferlen(b).valueOf();
  }), 4n);

  assert((await page.evaluate(async () => {
    let b = await alanStd.createEmptyBuffer(alanStd.storageBufferType(), 4);
    return alanStd.bufferid(b).valueOf();
  })).startsWith("buffer_"));

  assert.deepEqual(await page.evaluate(async () => {
    let b = await alanStd.createBufferInit(alanStd.storageBufferType(), [1, 2, 3, 4]);
    let v = await alanStd.readBuffer(b);
    return v.map((i) => i.valueOf());
  }), [1, 2, 3, 4]);

  assert.deepEqual(await page.evaluate(async () => {
    let b = await alanStd.createBufferInit(alanStd.storageBufferType(), [
      new alanStd.I32(1),
      new alanStd.I32(2),
      new alanStd.I32(3),
      new alanStd.I32(4)
    ]);
    let v = await alanStd.readBuffer(b);
    return v.map((i) => i.valueOf());
  }), [1, 2, 3, 4]);

  assert.deepEqual(await page.evaluate(async () => {
    let b = await alanStd.createBufferInit(alanStd.storageBufferType(), [1, 2, 3, 4]);
    await alanStd.replaceBuffer(b, [5, 6, 7, 8]);
    let v = await alanStd.readBuffer(b);
    return v.map((i) => i.valueOf());
  }), [5, 6, 7, 8]);

  assert.deepEqual(await page.evaluate(async () => {
    let b = await alanStd.createBufferInit(alanStd.storageBufferType(), [2, 2, 2, 2]);
    let plan = new alanStd.GPGPU(`
      @group(0)
      @binding(0)
      var<storage, read_write> vals: array<i32>;

      @compute
      @workgroup_size(1)
      fn main(@builtin(global_invocation_id) id: vec3<u32>) {
        vals[id.x] = vals[id.x] * i32(id.x);
      }
    `, [4, 1, 1], [[b]]);
    await alanStd.gpuRun(plan);
    let v = await alanStd.readBuffer(b);
    return v.map((i) => i.valueOf());
  }), [0, 2, 4, 6]);

  await context.close();
  await browser.close();
})();
