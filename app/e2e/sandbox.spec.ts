import { expect, test, type Page } from '@playwright/test';

// The sandbox is laid out for a desktop for now. Phone layout is not done.
test.skip(({ viewport }) => (viewport?.width ?? 0) < 1024, 'the sandbox is desktop-only for now');

async function openTemplate(page: Page, id: string) {
  await page.goto('/#/build');
  await page.getByTestId(`template-${id}`).click();
  await expect(page.getByTestId('build')).toBeVisible();
}

const verdict = (page: Page, goal: string) => page.getByTestId(`verdict-${goal}`);

async function addPart(page: Page, group: string, pack: string) {
  await page.getByRole('tab', { name: group, exact: true }).click();
  await page.getByTestId(`part-${pack}`).click();
}

test('Home links to the sandbox, and a template opens with its report', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('build-your-own').click();
  await expect(page).toHaveURL(/#\/build$/);
  await page.getByTestId('template-power-cut-proof').click();
  await expect(verdict(page, 'reach-away')).toHaveAttribute('data-status', 'pass');
  await expect(verdict(page, 'power-cut')).toHaveAttribute('data-status', 'fail');
  await expect(verdict(page, 'power-cut')).toContainText("Won't work");
  await expect(verdict(page, 'running-cost')).toHaveAttribute('data-status', 'pass');
  // The core's figure: 9 W for a year on the teaching tariff.
  await expect(page.getByTestId('yearly')).toContainText('₹459.90');
  await expect(page.getByTestId('ups-ups')).toContainText('about 302 minutes');
});

test('a failing goal is fixed by adding a part', async ({ page }) => {
  await openTemplate(page, 'power-cut-proof');
  await expect(verdict(page, 'power-cut')).toHaveAttribute('data-status', 'fail');
  await addPart(page, 'Power', 'dc-mini-ups');
  await page.getByTestId('feed-router').check();
  await page.getByTestId('tab-report').click();
  await expect(verdict(page, 'power-cut')).toHaveAttribute('data-status', 'pass');

  // A laptop with no Ethernet port, cabled to the router: a USB adapter gives it one.
  await openTemplate(page, 'macbook-air-as-server');
  await expect(verdict(page, 'run-media')).toHaveAttribute('data-status', 'fail');
  await page.getByTestId('node-server').click();
  await addPart(page, 'Adapters', 'usb-ethernet-adapter');
  await page.getByTestId('tab-report').click();
  await expect(verdict(page, 'run-media')).toHaveAttribute('data-status', 'pass');
});

test('what the data does not say is "can\'t tell", with the missing field named', async ({ page }) => {
  await openTemplate(page, 'pi-hole-whole-house');
  await expect(verdict(page, 'adblock-house')).toHaveAttribute('data-status', 'unknown');
  await expect(verdict(page, 'adblock-house')).toContainText("Can't tell yet");
  const unknown = page.getByTestId('unknown').first();
  await expect(unknown).toContainText('ISP router we know nothing about');
  const link = page.getByTestId('help-add').first();
  if (await link.count()) {
    // Only the pack and the field go in the link.
    const url = new URL((await link.getAttribute('href')) ?? '');
    expect([...url.searchParams.keys()].sort()).toEqual(['field', 'pack', 'template', 'title']);
  }
  // Choosing a router we do have data for settles it.
  await page.getByTestId('node-router').click();
  await page.getByTestId('device').selectOption('retail-router');
  await page.getByTestId('tab-report').click();
  await expect(verdict(page, 'adblock-house')).toHaveAttribute('data-status', 'pass');
});

test('a build from an empty board: router, laptop, cable, system, service, goal', async ({ page }) => {
  await page.goto('/#/build');
  await page.getByTestId('start-empty').click();
  await addPart(page, 'Network', 'retail-router');
  await addPart(page, 'Computers', 'old-laptop');
  await page.getByTestId('connect').click();
  await expect(page.getByTestId('connect-hint')).toBeVisible();
  await page.getByTestId('node-router-you-bought-and-control').click();
  await page.getByTestId('cable-ethernet').click();
  await page.getByTestId('node-old-laptop-used-as-a-server').click();
  await page.getByTestId('os').selectOption('linux-server');
  await page.getByTestId('service').selectOption('media-server');
  await page.getByTestId('service-add').click();
  await page.getByTestId('tab-goals').click();
  await page.getByTestId('goal-add').click();
  await page.getByTestId('tab-report').click();
  // Nothing to install the system on yet.
  await expect(verdict(page, 'run')).toHaveAttribute('data-status', 'fail');
  await page.getByTestId('node-old-laptop-used-as-a-server').click();
  await page.getByTestId('part').selectOption('ssd-500gb');
  await page.getByTestId('part-add').click();
  await page.getByTestId('tab-report').click();
  await expect(verdict(page, 'run')).toHaveAttribute('data-status', 'pass');
  await expect(page.getByTestId('fit-old-laptop-used-as-a-server')).toContainText('1324 of 8192 MB');

  // It is still there after a reload.
  await page.reload();
  await expect(verdict(page, 'run')).toHaveAttribute('data-status', 'pass');
});

test('a device the learner describes is unknown where they left it blank', async ({ page }) => {
  await page.goto('/#/build');
  await page.getByTestId('start-empty').click();
  await page.getByTestId('custom-open').click();
  await page.getByTestId('custom-label').fill('Office PC');
  await page.getByTestId('custom-arch').selectOption('x86_64');
  await page.getByTestId('custom-add').click();
  await page.getByTestId('os').selectOption('linux-server');
  await page.getByTestId('service').selectOption('media-server');
  await page.getByTestId('service-add').click();
  await page.getByTestId('tab-goals').click();
  await page.getByTestId('goal-add').click();
  await page.getByTestId('tab-report').click();
  await expect(verdict(page, 'run')).toHaveAttribute('data-status', 'unknown');
  await expect(page.getByTestId('unknown').filter({ hasText: 'compute.ram_gb' })).toBeVisible();
});

test('a shared link carries the build and leaves the prices behind', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await openTemplate(page, 'power-cut-proof');
  await page.getByTestId('node-server').click();
  await page.getByTestId('price').fill('18000');
  await page.getByTestId('tab-report').click();
  await expect(page.getByTestId('total')).toContainText('₹18,000.00');

  await page.getByTestId('copy-link').click();
  await expect(page.getByTestId('share-status')).toContainText('Link copied');
  const link = await page.evaluate(() => navigator.clipboard.readText());
  expect(link).toContain('#/build/import/');

  await page.goto('about:blank');
  await page.goto(link);
  await expect(page.getByTestId('build')).toBeVisible();
  await expect(page).not.toHaveURL(/import/);
  await expect(verdict(page, 'power-cut')).toHaveAttribute('data-status', 'fail');
  await expect(page.getByTestId('total')).toHaveCount(0);
});

test('a damaged link is refused with a reason', async ({ page }) => {
  await page.goto('/#/build/import/not-a-build');
  await expect(page.getByTestId('import-problems')).toContainText('could not be read');
});

test('the sandbox works when the browser refuses storage', async ({ page }) => {
  await page.addInitScript(() => {
    const refuse = () => {
      throw new DOMException('denied', 'SecurityError');
    };
    Object.defineProperty(window, 'localStorage', { get: refuse });
  });
  await page.goto('/#/build');
  await page.getByTestId('template-old-phone-tiny-server').click();
  await expect(page.getByTestId('storage-notice')).toBeVisible();
  await expect(verdict(page, 'run-sync')).toHaveAttribute('data-status', 'pass');
  await expect(verdict(page, 'run-dns')).toHaveAttribute('data-status', 'fail');
});
