#!/usr/bin/env node
// Phase 1 spike, throwaway. Measures both spikes in installed Chrome through
// playwright-core. Usage: node measure.mjs [terminal|trace] (default both).
// Low-end phone is approximated with CDP CPU throttling (6x) on desktop
// hardware. It is not a real phone.
import child_process from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import { build, createServer, preview } from 'vite';
import { chromium } from 'playwright-core';

const here = import.meta.dirname;
const only = process.argv[2];
const shots = process.env.SHOTS ?? path.join(here, 'dist');
const configFile = path.join(here, 'vite.config.js');
const out = {};

const browser = await chromium.launch({ channel: 'chrome', headless: true });

async function open(url, { throttle = 1, mobile = false } = {}) {
  const context = await browser.newContext(
    mobile
      ? { viewport: { width: 360, height: 740 }, deviceScaleFactor: 3, isMobile: true, hasTouch: true }
      : { viewport: { width: 1280, height: 800 } },
  );
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send('Network.enable');
  const net = { requests: 0, bytes: 0, byKind: {} };
  const urls = new Map();
  cdp.on('Network.requestWillBeSent', (e) => urls.set(e.requestId, e.request.url));
  cdp.on('Network.loadingFinished', (e) => {
    const u = urls.get(e.requestId) ?? '';
    const kind = u.includes('alpine-rootfs-flat')
      ? 'rootfs'
      : u.includes('alpine-state')
        ? 'state'
        : u.includes('alpine-fs.json')
          ? 'fsIndex'
          : u.includes('bios')
            ? 'bios'
            : 'pageAndLib';
    net.requests++;
    net.bytes += e.encodedDataLength;
    const k = (net.byKind[kind] ??= { requests: 0, bytes: 0 });
    k.requests++;
    k.bytes += e.encodedDataLength;
  });
  if (throttle > 1) await cdp.send('Emulation.setCPUThrottlingRate', { rate: throttle });
  page.on('pageerror', (e) => console.error('pageerror:', e.message));
  await page.goto(url);
  return { context, page, cdp, net };
}

function rendererRssMb() {
  // Largest Chrome renderer started by this script's browser.
  const ps = child_process.execSync('ps -axo rss=,command=').toString().split('\n');
  const rss = ps
    .filter((l) => l.includes('--type=renderer') && l.includes('playwright'))
    .map((l) => Number(l.trim().split(/\s+/)[0]));
  return rss.length ? Math.round(Math.max(...rss) / 1024) : null;
}

const gz = (file) => zlib.gzipSync(fs.readFileSync(file), { level: 9 }).length;
const br = (file) => zlib.brotliCompressSync(fs.readFileSync(file)).length;

if (!only || only === 'terminal') {
  const server = await createServer({ configFile, server: { port: 0 }, logLevel: 'error' });
  await server.listen();
  const base = `http://localhost:${server.httpServer.address().port}/terminal/`;
  const v86 = path.join(here, '../node_modules/v86/build');
  out.terminal = {
    engine: {
      'libv86.mjs gzip': gz(path.join(v86, 'libv86.mjs')),
      'v86.wasm gzip': gz(path.join(v86, 'v86.wasm')),
      'v86.wasm brotli': br(path.join(v86, 'v86.wasm')),
    },
    runs: [],
  };
  const runs = [
    { query: 'auto=1', throttle: 1 },
    { query: 'auto=1&state=256', throttle: 1 },
    { query: 'auto=1&state=128', throttle: 1 },
    { query: 'auto=1&state=128', throttle: 6 },
    { query: 'auto=1', throttle: 6 },
  ];
  for (const run of runs) {
    const { context, page, net } = await open(`${base}?${run.query}`, { throttle: run.throttle });
    try {
      await page.waitForFunction(() => window.__result, null, { timeout: 15 * 60 * 1000 });
      const result = await page.evaluate(() => window.__result);
      const rssMb = rendererRssMb();
      const key = result.readAfterSync.match(/PrivateKey = (\S+)/)?.[1] ?? '';
      result.readBeforeSync = result.readBeforeSync === result.readAfterSync ? 'same as after sync' : result.readBeforeSync;
      result.readAfterSync = `[Interface] with ${key.length}-char key, ${result.readAfterSync.length} bytes`;
      out.terminal.runs.push({ ...run, ...result, rendererRssMb: rssMb, net });
      await page.screenshot({ path: path.join(shots, `terminal-${result.mode}-x${run.throttle}.png`) });
    } catch (e) {
      out.terminal.runs.push({ ...run, error: e.message.split('\n')[0], net });
    }
    await context.close();
  }
  await server.close();
}

if (!only || only === 'trace') {
  // createServer above sets NODE_ENV=development, which would give a dev React build.
  process.env.NODE_ENV = 'production';
  await build({ configFile, logLevel: 'error' });
  const assets = path.join(here, 'dist/assets');
  out.traceBundle = Object.fromEntries(
    fs.readdirSync(assets).map((f) => [f, { bytes: fs.statSync(path.join(assets, f)).size, gzip: gz(path.join(assets, f)) }]),
  );
  // React Flow alone: same import as the spike, React left external.
  const lib = await build({
    configFile: false,
    logLevel: 'error',
    root: here,
    build: {
      write: false,
      lib: { entry: path.join(here, 'trace/xyflow-only.js'), formats: ['es'] },
      rollupOptions: { external: ['react', 'react-dom', 'react/jsx-runtime'] },
    },
  });
  out.reactFlowAlone = Object.fromEntries(
    lib[0].output.map((o) => {
      const body = Buffer.from(o.code ?? o.source);
      return [o.fileName, { bytes: body.length, gzip: zlib.gzipSync(body, { level: 9 }).length }];
    }),
  );

  const server = await preview({ configFile, preview: { port: 0 }, logLevel: 'error' });
  const url = `http://localhost:${server.httpServer.address().port}/trace/`;
  out.trace = [];
  for (const run of [
    { throttle: 1, mobile: false },
    { throttle: 1, mobile: true },
    { throttle: 6, mobile: true },
    { throttle: 20, mobile: true },
  ]) {
    const { context, page } = await open(url, run);
    await page.waitForFunction(() => window.__spike?.player.timeline);
    const total = await page.evaluate(() => window.__spike.player.timeline.total);
    // Proof that the throttle is applied: a fixed busy loop, and the script
    // cost of one playback frame (seek paints once).
    const busyLoopMs = await page.evaluate(() => {
      const t = performance.now();
      let x = 0;
      for (let i = 0; i < 3e7; i++) x += Math.sqrt(i);
      return Number((performance.now() - t).toFixed(1)) + (x < 0 ? 1 : 0);
    });
    const paintMs = await page.evaluate(() => {
      const p = window.__spike.player;
      const t = performance.now();
      for (let i = 0; i < 500; i++) p.seek((i / 500) * p.timeline.total);
      p.seek(0);
      return Number(((performance.now() - t) / 500).toFixed(3));
    });
    const idle = await page.evaluate(() => window.__spike.recordFrames(1500));
    const playing = await page.evaluate((ms) => {
      window.__spike.player.play();
      return window.__spike.recordFrames(ms);
    }, total);
    await page.screenshot({ path: path.join(shots, `trace-${run.mobile ? 'mobile' : 'desktop'}-x${run.throttle}.png`) });

    // Playback at 0.5x, while the board is panned and then zoomed.
    const box = await page.locator('.react-flow__pane').boundingBox();
    const cx = box.x + box.width / 2;
    const cy = box.y + box.height / 2;
    const during = page.evaluate((ms) => {
      const p = window.__spike.player;
      p.seek(0);
      p.speed = 0.5;
      p.play();
      return Promise.all([window.__spike.recordFrames(ms), window.__spike.recordPositionError(ms)]);
    }, 4000);
    const cdp = await context.newCDPSession(page);
    if (run.mobile) {
      const touch = (type, x, y) =>
        cdp.send('Input.dispatchTouchEvent', { type, touchPoints: type === 'touchEnd' ? [] : [{ x, y }] });
      await touch('touchStart', cx, cy);
      for (let i = 1; i <= 30; i++) await touch('touchMove', cx - i * 3, cy + Math.sin(i / 4) * 20);
      await touch('touchEnd', 0, 0);
      await cdp.send('Input.synthesizePinchGesture', { x: cx, y: cy, scaleFactor: 1.8, relativeSpeed: 400 });
    } else {
      await page.mouse.move(cx, cy);
      await page.mouse.down();
      for (let i = 1; i <= 30; i++) await page.mouse.move(cx - i * 5, cy + Math.sin(i / 4) * 30);
      await page.mouse.up();
      for (let i = 0; i < 8; i++) await page.mouse.wheel(0, -120);
    }
    const [panZoomFrames, position] = await during;
    const viewport = await page.evaluate(() => document.querySelector('.react-flow__viewport').style.transform);
    await page.screenshot({ path: path.join(shots, `trace-${run.mobile ? 'mobile' : 'desktop'}-x${run.throttle}-panzoom.png`) });
    out.trace.push({ ...run, busyLoopMs, paintMs, idle, playing, panZoomFrames, position, viewportAfter: viewport });
    await context.close();
  }
  await server.close();
}

await browser.close();
console.log(JSON.stringify(out, null, 2));
process.exit(0);
