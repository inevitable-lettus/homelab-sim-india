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
  for (const path of ['/', '/#/mission/dial-out', '/#/settings', '/#/credits', '/#/concept/cgnat-no-inbound-mapping']) {
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
