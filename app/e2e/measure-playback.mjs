// One-off measurement, not a test: frame times while a trace plays, with the
// CPU slowed down. Run after `npm run build`, with `npm run preview -w app`
// serving on port 4173:  node e2e/measure-playback.mjs [throttle]
import { chromium } from '@playwright/test';

const rate = Number(process.argv[2] ?? 6);
const browser = await chromium.launch({ channel: 'chrome' });
for (const [name, viewport, mission] of [
  ['desktop 1280x800', { width: 1280, height: 800 }, 'dial-out'],
  ['phone 360x740 @3x', { width: 360, height: 740 }, 'dial-out'],
]) {
  const context = await browser.newContext({ viewport, deviceScaleFactor: name.startsWith('phone') ? 3 : 1, hasTouch: name.startsWith('phone') });
  const page = await context.newPage();
  await page.goto(`http://127.0.0.1:4173/#/mission/${mission}`);
  await page.getByTestId('join-server').check();
  await page.getByTestId('join-phone').check();
  const tab = page.getByTestId('tab-board');
  if (await tab.isVisible()) await tab.click();
  const cdp = await context.newCDPSession(page);
  await cdp.send('Emulation.setCPUThrottlingRate', { rate });
  await page.getByTestId('run').click();
  const frames = await page.evaluate(
    () =>
      new Promise((done) => {
        const gaps = [];
        let last = performance.now();
        const stop = last + 6000;
        const tick = (now) => {
          gaps.push(now - last);
          last = now;
          if (now < stop) requestAnimationFrame(tick);
          else done(gaps);
        };
        requestAnimationFrame(tick);
      }),
  );
  const sorted = [...frames].sort((a, b) => a - b);
  const at = (q) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * q))].toFixed(1);
  const fps = (1000 / (frames.reduce((a, b) => a + b, 0) / frames.length)).toFixed(1);
  console.log(`${name}, ${rate}x CPU: ${fps} fps, median ${at(0.5)} ms, p95 ${at(0.95)} ms, worst ${at(1)} ms, ${frames.length} frames`);
  await context.close();
}
await browser.close();
