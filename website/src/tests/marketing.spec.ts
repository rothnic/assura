import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const widths = [360, 390, 430, 768, 1024, 1440];
const themes = ['light', 'dark'] as const;
const marketingRoutes = [
  { path: '/', heading: 'Catch project drift before review.' },
  { path: '/compare/ls-lint/', heading: 'A faster path from naming checks to agent-ready project validation.' },
  { path: '/performance/', heading: 'Fast checks keep agent work moving.' },
  { path: '/ai-coding-agent-guardrails/', heading: 'Guide the repair before a late gate forces it.' },
  { path: '/about/', heading: 'Built to make AI-assisted work easier to trust.' },
  { path: '/project-review/', heading: 'See what this branch changed before review starts.' },
  { path: '/agent-onboarding/', heading: 'Give the agent a baseline without teaching it to guess.' },
  { path: '/repository-validation/', heading: 'Validate the project from the top down.' },
  { path: '/project-intelligence/', heading: 'Turn local project facts into better agent context.' },
  { path: '/examples/', heading: 'Start from a project shape that resembles yours.' },
  { path: '/insights/benchmark-methodology/', heading: 'Measure equivalent work before comparing speed.' },
  { path: '/case-studies/dogfooding-assura/', heading: 'The repository is the first production fixture.' },
  { path: '/changelog/', heading: 'Release evidence, not just release notes.' },
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

test('example output CTA connects project policy to pass and fail paths', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await page.getByRole('link', { name: 'See how rules apply' }).click();
  await expect(page.getByRole('heading', { name: '.assura/config.yml' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Project tree' })).toBeVisible();
  await expect(page.getByLabel('Pass').first()).toBeVisible();
  await expect(page.getByLabel('Blocking violation').first()).toBeVisible();
});

test('performance CTA lands on the measured project cohort', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'How we measured it' }).click();
  await expect(page).toHaveURL(/\/performance\/#measured-comparison$/);
  await expect(page.getByRole('heading', { name: 'Faster than native LS-Lint in all eight cold comparisons.' })).toBeVisible();
  await expect(page.locator('.policy-breadth-card')).toHaveCount(1);
  await expect(page.locator('.policy-config-pane')).toHaveCount(2);
  await expect(page.getByText('Built-in + project-defined', { exact: true })).toBeVisible();
  await expect(page.getByText('One wildcard package scope', { exact: true })).toBeVisible();
  await expect(page.getByText('Capability example · not timed', { exact: true })).toBeVisible();
  await expect(page.locator('.benchmark-card')).toHaveCount(3);
  await expect(page.locator('.benchmark-config')).toHaveCount(3);
  await expect(page.locator('.benchmark-matrix-row:not(.benchmark-matrix-head)')).toHaveCount(8);
  await expect(page.getByText('1,501 files', { exact: true })).toBeVisible();
  await expect(page.getByText('801 rules', { exact: true })).toBeVisible();
});

for (const colorScheme of themes) {
  for (const width of [360, 390, 768, 1440]) {
    test(`${colorScheme} performance policy at ${width}px stays contained`, async ({ page }, testInfo) => {
      await page.emulateMedia({ colorScheme });
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/performance/#regression-cases');

      const comparison = page.locator('.policy-breadth-card');
      await expect(comparison).toBeVisible();
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      const codeOverflow = await comparison.locator('pre').evaluateAll((blocks) =>
        blocks.map((block) => block.scrollWidth - block.clientWidth),
      );
      const panes = await comparison.locator('.policy-config-pane').evaluateAll((items) =>
        items.map((item) => {
          const rect = item.getBoundingClientRect();
          return { x: Math.round(rect.x), y: Math.round(rect.y) };
        }),
      );
      expect(overflow).toBe(0);
      expect(codeOverflow).toEqual([0, 0]);
      if (width <= 768) {
        expect(panes[0].x).toBe(panes[1].x);
        expect(panes[0].y).toBeLessThan(panes[1].y);
      } else {
        expect(panes[0].x).toBeLessThan(panes[1].x);
        expect(panes[0].y).toBe(panes[1].y);
      }
      await comparison.screenshot({
        path: testInfo.outputPath(`performance-policy-${colorScheme}-${width}.png`),
      });
    });
  }
}

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

test('marketing pages do not publish broken internal links', async ({ page, request }) => {
  const links = new Set<string>();
  for (const route of marketingRoutes) {
    await page.goto(route.path);
    const hrefs = await page.locator('a[href^="/"]').evaluateAll((anchors) =>
      anchors.map((anchor) => anchor.getAttribute('href')).filter((href): href is string => Boolean(href)),
    );
    hrefs.forEach((href) => links.add(href));
  }

  for (const href of links) {
    if (href.startsWith('/#')) continue;
    const response = await request.get(href);
    expect(response.status(), `Expected ${href} to resolve`).toBeLessThan(400);
  }
});
