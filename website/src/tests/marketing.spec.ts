import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const widths = [360, 390, 430, 768, 1024, 1440];
const themes = ['light', 'dark'] as const;
const marketingRoutes = [
  { path: '/', heading: 'Catch project drift before review.' },
  { path: '/compare/ls-lint/', heading: 'A faster path from naming checks to agent-ready project validation.' },
  { path: '/performance/', heading: 'Fast enough for the check. Warm enough for the loop.' },
  { path: '/ai-coding-agent-guardrails/', heading: 'Guide the repair before a late gate forces it.' },
  { path: '/about/', heading: 'Built to make AI-assisted work easier to trust.' },
];

for (const colorScheme of themes) {
  for (const width of widths) {
    test(`${colorScheme} landing at ${width}px has no horizontal overflow`, async ({ page }, testInfo) => {
      await page.emulateMedia({ colorScheme });
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/');
      await expect(page.locator('#hero-title')).toHaveText('Catch project drift before review.');

      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow).toBe(0);
      await page.screenshot({
        path: testInfo.outputPath(`landing-${colorScheme}-${width}.png`),
        fullPage: true,
      });
    });
  }
}

test('agent setup dialog is keyboard dismissible and restores focus', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const trigger = page.getByRole('link', { name: 'Start with your agent' }).first();
  await trigger.click();

  const dialog = page.getByRole('dialog', { name: 'Start with one agent instruction.' });
  await expect(dialog).toBeVisible();
  await expect(page.getByRole('button', { name: 'Close setup dialog' })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

test('short mobile view keeps the project review visible', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 640 });
  await page.goto('/');
  await expect(page.getByText('$ assura review').first()).toBeVisible();
});

test('setup actions retain a no-JavaScript onboarding destination', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('link', { name: 'Start with your agent' }).first()).toHaveAttribute('href', '#onboard');
});

test('landing and setup dialog pass automated accessibility checks', async ({ page }) => {
  await page.goto('/');
  const landing = await new AxeBuilder({ page }).analyze();
  expect(landing.violations).toEqual([]);

  await page.getByRole('link', { name: 'Start with your agent' }).first().click();
  const dialog = page.getByRole('dialog', { name: 'Start with one agent instruction.' });
  await expect(dialog).toBeVisible();
  const dialogAudit = await new AxeBuilder({ page }).include('#agent-setup-dialog').analyze();
  expect(dialogAudit.violations).toEqual([]);
});

test('reduced motion disables smooth scrolling', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/');
  const scrollBehavior = await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior);
  expect(scrollBehavior).toBe('auto');
});

test('marketing commands stay on the current public CLI surface', async ({ page }) => {
  await page.goto('/');
  const text = await page.locator('body').innerText();
  expect(text).toContain('assura review');
  expect(text).toContain('assura explain');
  expect(text).not.toContain('assura review --base');
  expect(text).not.toContain('assura review --path');
});

test('technical docs remain reachable', async ({ page }) => {
  const response = await page.goto('/guides/quickstart/');
  expect(response?.status()).toBe(200);
  await expect(page.locator('main')).toBeVisible();
});

test('P1 routes expose unique canonical metadata and structured data', async ({ page }) => {
  const titles = new Set<string>();
  const descriptions = new Set<string>();

  for (const route of marketingRoutes) {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(route.path);
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(route.heading);
    await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', `https://assura.dev${route.path}`);
    await expect(page.locator('meta[property="og:image"]')).toHaveAttribute('content', /\/social\/.*\.webp$/);
    const title = await page.title();
    const description = await page.locator('meta[name="description"]').getAttribute('content');
    expect(titles.has(title)).toBe(false);
    expect(descriptions.has(description || '')).toBe(false);
    titles.add(title);
    descriptions.add(description || '');
    const jsonLd = await page.locator('script[type="application/ld+json"]').textContent();
    expect(() => JSON.parse(jsonLd || '')).not.toThrow();
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBe(0);
    const audit = await new AxeBuilder({ page }).analyze();
    expect(audit.violations).toEqual([]);
  }
});

test('robots policy and sitemap discovery endpoints are available', async ({ page, request }) => {
  const robots = await request.get('/robots.txt');
  expect(robots.ok()).toBe(true);
  expect(await robots.text()).toContain('Sitemap: https://assura.dev/sitemap-index.xml');
  await page.goto('/');
  await expect(page.locator('link[rel="sitemap"]')).toHaveAttribute('href', '/sitemap-index.xml');
  await expect(page.locator('#robots-policy')).toHaveAttribute('content', 'index, follow');
});

test('high-value CTA emits a named analytics event', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => {
    (window as Window & { capturedCta?: unknown }).capturedCta = null;
    window.addEventListener('assura:cta', (event) => {
      (window as Window & { capturedCta?: unknown }).capturedCta = (event as CustomEvent).detail;
    }, { once: true });
  });
  await page.getByRole('link', { name: 'Start with your agent' }).first().click();
  const event = await page.evaluate(() => (window as Window & { capturedCta?: unknown }).capturedCta);
  expect(event).toMatchObject({ name: 'setup_open', path: '/' });
});
