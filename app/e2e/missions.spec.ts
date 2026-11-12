import { existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { saveForward, setAnswer } from '../src/lib/world';
import { bundle, grade, missionWorld, stepsOf } from './core';

/** On a phone the mission shows one panel at a time. */
async function show(page: Page, tab: 'brief' | 'board' | 'findings') {
  const button = page.getByTestId(`tab-${tab}`);
  if (await button.isVisible()) await button.click();
}

async function runToEnd(page: Page) {
  await show(page, 'board');
  await page.getByTestId('skip').click();
  await expect(page.getByTestId('result').or(page.getByTestId('debrief')).first()).toBeVisible();
}

async function loggedSteps(page: Page) {
  return page
    .getByTestId('step')
    .evaluateAll((items) => items.map((item) => ({ at: item.getAttribute('data-at'), reason: item.getAttribute('data-reason') })));
}

test('the landing page starts Mission 1 without a signup', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toContainText('still doesn');
  await page.getByTestId('start').click();
  await expect(page).toHaveURL(/#\/mission\/which-address$/);
  await expect(page.getByTestId('goals')).toBeVisible();
});

test('Open a door: the obvious attempt fails at the router, and a forward fixes it', async ({ page }) => {
  const start = missionWorld('open-a-door');
  await page.goto('/#/mission/open-a-door');

  // Nothing is judged before the learner has run it.
  await expect(page.getByTestId('result')).toHaveCount(0);
  await expect(page.getByTestId('finding')).toHaveCount(0);

  await runToEnd(page);
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'dropped');
  await expect(page.getByTestId('result')).toContainText('Stopped at the router');
  expect(await loggedSteps(page)).toEqual(stepsOf(grade('open-a-door', start), 'open-jellyfin'));
  await expect(page.getByTestId('debrief')).toHaveCount(0);

  await show(page, 'brief');
  await page.getByTestId('add-forward').click();
  await page.getByLabel('Port on the router').fill('8096');
  await page.getByLabel('Send to this home address').fill('192.168.1.20');
  await page.getByLabel('Port on that device').fill('8096');
  await page.getByTestId('save-forward').click();

  // A changed setup has not been run yet, so the old verdict is gone.
  await show(page, 'board');
  await expect(page.getByTestId('result')).toHaveCount(0);

  await runToEnd(page);
  await expect(page.getByTestId('debrief')).toBeVisible();
  await expect(page.getByTestId('debrief')).toContainText('Mission complete');

  const solved = saveForward(start, 'home-router', { id: 'x', enabled: true, proto: 'tcp', external_port: 8096, to_ip: '192.168.1.20', internal_port: 8096 });
  const result = grade('open-a-door', solved);
  expect(result.status).toBe('pass');
  expect(await loggedSteps(page)).toEqual(stepsOf(result, 'open-jellyfin'));

  await show(page, 'findings');
  await expect(page.getByTestId('finding').first()).toContainText('In this model');
});

test('The wall: the packet stops at the ISP, and only the right answers finish it', async ({ page }) => {
  const start = missionWorld('the-wall');
  await page.goto('/#/mission/the-wall');
  await runToEnd(page);
  await expect(page.getByTestId('result')).toContainText("Stopped at the ISP's carrier NAT");
  expect(await loggedSteps(page)).toEqual(stepsOf(grade('the-wall', start), 'open-jellyfin'));

  await show(page, 'brief');
  await page.getByTestId('answer-blocked-at-home-router').check();
  await expect(page.getByTestId('goal-blocked-at')).toHaveAttribute('data-status', 'fail');
  await page.getByTestId('answer-blocked-at-isp').check();
  await expect(page.getByTestId('goal-blocked-at')).toHaveAttribute('data-status', 'pass');
  await page.getByTestId('answer-line-type-different').check();

  const answered = setAnswer(setAnswer(start, 'blocked-at', 'isp'), 'line-type', 'different');
  expect(grade('the-wall', answered).status).toBe('pass');
  await show(page, 'board');
  await expect(page.getByTestId('debrief')).toBeVisible();
});

test('reduced motion: nothing autoplays, and every step can be reached by hand', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/#/mission/the-wall');
  await show(page, 'board');
  const expected = stepsOf(grade('the-wall', missionWorld('the-wall')), 'open-jellyfin');
  await page.getByTestId('run').click();
  await expect(page.getByTestId('step')).toHaveCount(1);
  for (let shown = 1; shown < expected.length; shown += 1) {
    await page.getByTestId('run').click();
    await expect(page.getByTestId('step')).toHaveCount(shown + 1);
  }
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'dropped');
  expect(await loggedSteps(page)).toEqual(expected);
});

test('no page scrolls sideways, and every explanation has a page', async ({ page }) => {
  for (const path of ['/', '/#/mission/dial-out', '/#/mission/write-the-tunnel', '/#/settings', '/#/credits', '/#/concept/cgnat-no-inbound-mapping']) {
    await page.goto(path);
    await expect(page.locator('h1, h2').first()).toBeVisible();
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow, path).toBeLessThanOrEqual(0);
  }
  expect(bundle.missions['dial-out']).toBeTruthy();
});

test('pressing Run plays the trace to its end on its own', async ({ page }) => {
  await page.goto('/#/mission/the-wall');
  await show(page, 'board');
  await page.getByTestId('run').click();
  await expect(page.getByTestId('run')).toContainText('Pause');
  await expect(page.getByTestId('result')).toBeVisible({ timeout: 15_000 });
  await expect(page.getByTestId('run')).toContainText('Run again');
  // The packet rests on the hop that dropped it.
  await expect(page.locator('.react-flow__node[data-id="isp"]')).toHaveAttribute('data-live', 'drop');
});

test('the board works from the keyboard', async ({ page, isMobile }) => {
  test.skip(isMobile, 'keyboard use is a desktop path');
  await page.goto('/#/mission/open-a-door');
  const router = page.getByRole('button', { name: /^Home router/ });
  await router.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('inspector')).toBeVisible();
  await expect(page.getByTestId('inspector')).toContainText('203.0.113.50');
  await page.keyboard.press('Escape');
  await expect(page.getByTestId('inspector')).toHaveCount(0);
});

test('Write the tunnel: the files decide, from the editor', async ({ page }) => {
  const world = missionWorld('write-the-tunnel');
  const now = 'Straight away';
  const later = 'Ten minutes later';
  const runAttempt = async (label: string) => {
    await show(page, 'board');
    await page.getByRole('radio', { name: label }).click();
    await runToEnd(page);
  };
  await page.goto('/#/mission/write-the-tunnel');

  // With no files the server has no tunnel, and the hub has nowhere to send.
  await runToEnd(page);
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'dropped');
  expect(await loggedSteps(page)).toEqual(stepsOf(grade('write-the-tunnel', world), 'reach-now'));

  // The starter file is not a config yet. The core says where.
  await show(page, 'brief');
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('file-wg0-conf')).toHaveAttribute('data-status', 'fail');
  await expect(page.getByTestId('problems-wg0-conf')).toContainText('Line 4');
  await expect(page.getByTestId('file-server-public-key')).toHaveAttribute('data-status', 'fail');

  // Make a key pair. The public key shown is the one the core derives.
  await page.getByTestId('make-keys').click();
  const privateKey = (await page.getByTestId('private-key').textContent()) ?? '';
  const publicKey = (await page.getByTestId('public-key').textContent()) ?? '';
  expect(privateKey).toMatch(/^[A-Za-z0-9+/]{43}=$/);
  await page.getByTestId('place-keys').click();
  await expect(page.getByTestId('editor-wg0-conf')).toHaveValue(new RegExp(`PrivateKey = ${privateKey.replace(/[+/]/g, '\\$&')}`));
  await expect(page.getByTestId('editor-server-public-key')).toHaveValue(`${publicKey}\n`);

  // A config with no keepalive: fine at first, dead ten minutes later.
  const hub = bundle.missions['write-the-tunnel']!.world.nodes.find((n) => n.id === 'hub');
  const hubKey = hub && 'tunnels' in hub ? hub.tunnels?.[0]?.public_key : undefined;
  const conf = `[Interface]\nPrivateKey = ${privateKey}\nAddress = 10.8.0.2/24\n\n[Peer]\nPublicKey = ${hubKey}\nEndpoint = 192.0.2.50:51820\nAllowedIPs = 10.8.0.0/24\n`;
  await page.getByTestId('editor-wg0-conf').fill(conf);
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('file-wg0-conf')).toHaveAttribute('data-status', 'pass');
  await expect(page.getByTestId('file-server-public-key')).toHaveAttribute('data-status', 'pass');

  const files = { '/root/wg0.conf': conf, '/root/publickey': `${publicKey}\n` };
  await runAttempt(now);
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'delivered');
  await runAttempt(later);
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'dropped');
  expect(await loggedSteps(page)).toEqual(stepsOf(grade('write-the-tunnel', world, files), 'reach-later'));
  await expect(page.getByTestId('debrief')).toHaveCount(0);

  // A public key from another pair: the core refuses it, and so does the hub.
  await show(page, 'brief');
  await page.getByTestId('editor-server-public-key').fill('hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTmo=');
  await expect(page.getByTestId('file-wg0-conf')).toHaveAttribute('data-status', 'waiting');
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('file-server-public-key')).toHaveAttribute('data-status', 'fail');
  await runAttempt(now);
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'dropped');

  // The right key and a keepalive: both attempts get through.
  await show(page, 'brief');
  await page.getByTestId('editor-server-public-key').fill(publicKey);
  await page.getByTestId('editor-wg0-conf').fill(`${conf}PersistentKeepalive = 25\n`);
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('file-wg0-conf')).toHaveAttribute('data-status', 'pass');
  await runAttempt(now);
  await runAttempt(later);
  await expect(page.getByTestId('debrief')).toContainText('Mission complete');
  const solved = grade('write-the-tunnel', world, { '/root/wg0.conf': `${conf}PersistentKeepalive = 25\n`, '/root/publickey': publicKey });
  expect(solved.status).toBe('pass');
  expect(await loggedSteps(page)).toEqual(stepsOf(solved, 'reach-later'));

  // The private key stays in this browser's storage and nowhere else.
  const saved = await page.evaluate(() => localStorage.getItem('homelab-sim:v1') ?? '');
  expect(saved).toContain(privateKey);
});

test('Write the tunnel: the same files, written in the real terminal', async ({ page, isMobile }, testInfo) => {
  test.skip(isMobile, 'the terminal is an opt-in extra on phones, and is checked on desktop');
  test.skip(!existsSync(resolve('public/guest/manifest.json')), 'this build has no guest image (tools/guest/build.sh)');
  test.setTimeout(120_000);
  const screen = page.getByTestId('terminal-screen').locator('.xterm-rows');
  const type = async (command: string, expect_: string | RegExp) => {
    await page.keyboard.type(`${command}\n`);
    await expect(screen).toContainText(expect_, { timeout: 30_000 });
  };
  const runAttempt = async (label: string) => {
    await page.getByRole('radio', { name: label }).click();
    await runToEnd(page);
  };

  await page.goto('/#/mission/write-the-tunnel');
  await expect(page.getByTestId('terminal-start')).toContainText('MB');
  await page.getByTestId('terminal-start').click();
  await expect(screen).toContainText('server:~#', { timeout: 60_000 });
  await page.getByTestId('terminal-screen').click();

  // The starter file from the editor is already there, and wg is installed.
  await type('grep -c Interface /root/wg0.conf; echo starter-$?', 'starter-0');
  await type('umask 077; wg genkey | tee privatekey | wg pubkey > publickey; echo keys-$?', 'keys-0');
  const hubKey = 'W9J3Hqm8EgeN6UewWxIP6ccD2CQZ8cC6fWREezgTW1U=';
  await type(
    `printf '[Interface]\\nPrivateKey = %s\\nAddress = 10.8.0.2/24\\n\\n[Peer]\\nPublicKey = ${hubKey}\\nEndpoint = 192.0.2.50:51820\\nAllowedIPs = 10.8.0.0/24\\n' "$(cat privatekey)" > wg0.conf; echo conf-$?`,
    'conf-0',
  );

  // Checking reads the guest's files into the page. No keepalive yet.
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('file-wg0-conf')).toHaveAttribute('data-status', 'pass');
  await expect(page.getByTestId('file-server-public-key')).toHaveAttribute('data-status', 'pass');
  await expect(page.getByTestId('editor-wg0-conf')).toHaveValue(/Endpoint = 192\.0\.2\.50:51820/);
  await runAttempt('Straight away');
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'delivered');
  await runAttempt('Ten minutes later');
  await expect(page.getByTestId('result')).toHaveAttribute('data-outcome', 'dropped');
  await expect(page.getByTestId('debrief')).toHaveCount(0);

  // Fix it in the terminal, check again, and both attempts get through.
  await page.getByTestId('terminal-screen').click();
  await type('echo PersistentKeepalive = 25 >> wg0.conf; echo fixed-$?', 'fixed-0');
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('editor-wg0-conf')).toHaveValue(/PersistentKeepalive = 25/);
  await runAttempt('Straight away');
  await runAttempt('Ten minutes later');
  await expect(page.getByTestId('debrief')).toContainText('Mission complete');

  // A file deleted in the terminal is a missing file, not a pass.
  await page.getByTestId('terminal-screen').click();
  await type('rm publickey; echo gone-$?', 'gone-0');
  await page.getByTestId('check-files').click();
  await expect(page.getByTestId('file-server-public-key')).toHaveAttribute('data-status', 'fail');
  await page.screenshot({ path: testInfo.outputPath('terminal.png'), fullPage: true });
});
