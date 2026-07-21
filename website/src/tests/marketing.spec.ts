import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';

const assuraPolicyFixture = readFileSync(
  new URL('../data/config-examples/agentic-monorepo.yml', import.meta.url),
  'utf8',
).trimEnd();

const lsLintPolicyFixture = readFileSync(
  new URL('../data/config-examples/agentic-monorepo.ls-lint.yml', import.meta.url),
  'utf8',
).trimEnd();

const homepagePolicyFixture = readFileSync(
  new URL('../data/config-examples/homepage-agentic-project.yml', import.meta.url),
  'utf8',
).trimEnd();

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
  const trigger = page.getByRole('link', { name: /with your agent/ }).first();
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
  await expect(page.locator('.config-line').filter({ hasText: 'agent-doc:' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: './**/:' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: '.ts: kebab-case | max_lines:500' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: 'AGENTS.md: exists:1 | $agent-doc' }).first()).toBeVisible();
  const configMarkers = await page.locator('.policy-panel .rule-marker').allTextContents();
  const treeMarkers = await page.locator('.tree-panel .rule-marker').allTextContents();
  expect(new Set(configMarkers)).toEqual(new Set(treeMarkers));
  await expect(page.getByLabel('Pass').first()).toBeVisible();
  await expect(page.getByLabel('Blocking violation').first()).toBeVisible();
});

test('compact policy links to the complete monorepo example through optional disclosure', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 844 });
  await page.goto('/#review-output');
  const disclosure = page.locator('.policy-demo-more');
  const link = disclosure.getByRole('link', { name: 'Open the complete monorepo policy' });
  await expect(disclosure).not.toHaveAttribute('open', '');
  await expect(link).toBeHidden();
  await disclosure.getByText('What the compact example leaves out').click();
  await expect(link).toBeVisible();
  await expect(link).toHaveAttribute('href', '/performance/#monorepo-policy');
  expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

for (const width of [320, 360, 390]) {
  test(`homepage policy is fully visible without nested scrolling at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto('/#review-output');
    const policy = page.locator('.policy-panel');
    await expect(policy).toBeVisible();
    const overflow = await policy.locator('pre').evaluate((element) => ({
      horizontal: element.scrollWidth - element.clientWidth,
      vertical: element.scrollHeight - element.clientHeight,
    }));
    expect(overflow).toEqual({ horizontal: 0, vertical: 0 });
    const markerGaps = await policy.locator('.config-line:has(> .rule-marker)').evaluateAll((lines) =>
      lines.map((line) => {
        const content = line.querySelector('.config-content')?.getBoundingClientRect();
        const marker = line.querySelector(':scope > .rule-marker')?.getBoundingClientRect();
        return Math.round((marker?.left ?? 0) - (content?.right ?? 0));
      }),
    );
    expect(Math.min(...markerGaps)).toBeGreaterThanOrEqual(4);
    const rendered = await policy.locator('.config-content').allTextContents();
    expect(rendered.join('\n')).toBe(homepagePolicyFixture);
  });
}

test('performance CTA lands on the measured project cohort', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'How we measured it' }).click();
  await expect(page).toHaveURL(/\/performance\/#measured-comparison$/);
  await expect(page.getByRole('heading', { name: 'Faster than native LS-Lint in all eight cold comparisons.' })).toBeVisible();
  await expect(page.locator('.policy-breadth-card')).toHaveCount(1);
  await expect(page.locator('.policy-wipe-layer')).toHaveCount(2);
  await expect(page.getByRole('slider')).toHaveCount(0);
  await expect(page.getByText('Compare shared filesystem rules first, then the policy Assura adds.', { exact: true })).toBeVisible();
  await expect(page.locator('.policy-coverage-row')).toHaveCount(8);
  await expect(page.getByText('Quality policy example', { exact: true })).toBeVisible();
  await expect(page.locator('.benchmark-card')).toHaveCount(3);
  await expect(page.locator('.benchmark-config')).toHaveCount(3);
  await expect(page.locator('.benchmark-matrix-row:not(.benchmark-matrix-head)')).toHaveCount(8);
  await expect(page.getByText('1,501 files', { exact: true })).toBeVisible();
  await expect(page.getByText('801 rules', { exact: true })).toBeVisible();
  await expect(page.getByText('Download current JSON')).toHaveCount(0);
});

test('footer groups product, resources, and creator links on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const footer = page.locator('.site-footer');
  await expect(footer.getByRole('heading', { name: 'Product' })).toBeVisible();
  await expect(footer.getByRole('heading', { name: 'Resources' })).toBeVisible();
  await expect(footer.getByRole('heading', { name: 'Connect' })).toBeVisible();
  await expect(footer.getByRole('link', { name: 'Nick Roth' })).toHaveAttribute('href', 'https://nickroth.com/');
  await expect(footer.getByRole('link', { name: 'LinkedIn' })).toBeVisible();
  const navigationBox = await footer.getByRole('navigation').boundingBox();
  const bylineBox = await footer.locator('.footer-byline').boundingBox();
  expect(bylineBox?.y).toBeGreaterThan((navigationBox?.y ?? 0) + (navigationBox?.height ?? 0));
  expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

for (const width of [320, 360, 390]) {
  test(`mobile performance evidence stays aligned at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto('/');
    const proof = page.locator('.performance-proof');
    const proofBox = await proof.boundingBox();
    const metrics = await proof.locator(':scope > div:not(.proof-meta)').evaluateAll((items) =>
      items.map((item) => {
        const rect = item.getBoundingClientRect();
        return { x: Math.round(rect.x), y: Math.round(rect.y), width: Math.round(rect.width) };
      }),
    );
    const metaItems = await proof.locator('.proof-meta > *').evaluateAll((items) =>
      items.map((item) => Math.round(item.getBoundingClientRect().y)),
    );
    const metricText = await proof.locator(':scope > div:not(.proof-meta)').evaluateAll((items) =>
      items.map((item) => {
        const label = item.querySelector('span')?.getBoundingClientRect();
        const value = item.querySelector('strong')?.getBoundingClientRect();
        const description = item.querySelector('small')?.getBoundingClientRect();
        return {
          labelY: Math.round(label?.y ?? 0),
          valueY: Math.round(value?.y ?? 0),
          descriptionHeight: Math.round(description?.height ?? 0),
        };
      }),
    );
    expect(metrics[0].y).toBe(metrics[1].y);
    expect(metrics[0].width).toBe(metrics[1].width);
    expect(metaItems[0]).toBe(metaItems[1]);
    expect(metricText[0].labelY).toBe(metricText[1].labelY);
    expect(metricText[0].valueY).toBe(metricText[1].valueY);
    expect(metricText[0].descriptionHeight).toBe(metricText[1].descriptionHeight);
    expect(metricText[0].descriptionHeight).toBeLessThanOrEqual(width < 390 ? 52 : 34);
    expect(proofBox?.height).toBeLessThan(width === 320 ? 230 : 210);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
  });

  test(`mobile lifecycle cards stay grouped at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await page.goto('/');
    const cards = await page.locator('.lifecycle-card').evaluateAll((items) =>
      items.map((item) => {
        const heading = item.querySelector('div')?.getBoundingClientRect();
        const body = item.querySelector('p')?.getBoundingClientRect();
        const card = item.getBoundingClientRect();
        return {
          gap: Math.round((body?.top ?? 0) - (heading?.bottom ?? 0)),
          height: Math.round(card.height),
        };
      }),
    );
    expect(cards).toHaveLength(3);
    expect(Math.max(...cards.map((card) => card.gap))).toBeLessThanOrEqual(16);
    expect(Math.max(...cards.map((card) => card.height))).toBeLessThan(190);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
  });
}

test('onboarding distinguishes applied recommendations from undecided policy', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/#onboard');
  const onboarding = page.locator('#onboard');
  await expect(onboarding.getByRole('heading', { name: 'Detects and applies' })).toBeVisible();
  await expect(onboarding.getByText('editable policy added to the project')).toBeVisible();
  await expect(onboarding.getByRole('heading', { name: 'Leaves undecided' })).toBeVisible();
  await expect(onboarding.getByText('language and framework rules')).toBeVisible();
});

test('mobile execution layers use compact aligned markers', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const rows = await page.locator('.layer-row').evaluateAll((items) =>
    items.map((item) => {
      const marker = item.firstElementChild?.getBoundingClientRect();
      const content = item.lastElementChild?.getBoundingClientRect();
      const row = item.getBoundingClientRect();
      return {
        markerX: Math.round(marker?.x ?? 0),
        contentX: Math.round(content?.x ?? 0),
        height: Math.round(row.height),
      };
    }),
  );
  expect(rows).toHaveLength(4);
  expect(new Set(rows.map((row) => row.markerX)).size).toBe(1);
  expect(new Set(rows.map((row) => row.contentX)).size).toBe(1);
  expect(Math.max(...rows.map((row) => row.contentX - row.markerX))).toBeLessThan(45);
  expect(Math.max(...rows.map((row) => row.height))).toBeLessThan(150);
});

for (const colorScheme of themes) {
  for (const width of [320, 360, 390, 768, 1440]) {
    test(`${colorScheme} performance policy at ${width}px stays contained`, async ({ page }, testInfo) => {
      await page.emulateMedia({ colorScheme });
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/performance/#regression-cases');

      const comparison = page.locator('.policy-breadth-card');
      await expect(comparison).toBeVisible();
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow).toBe(0);
      for (const view of ['Assura', 'LS-Lint']) {
        await comparison.getByRole('tab', { name: view }).click();
        const codeOverflow = await comparison
          .locator('.policy-wipe-layer:not([hidden]) .policy-code')
          .evaluate((block) => block.scrollWidth - block.clientWidth);
        expect(codeOverflow).toBe(0);
        expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
      }
      await comparison.screenshot({
        path: testInfo.outputPath(`performance-policy-${colorScheme}-${width}.png`),
      });
    });
  }
}

test('performance policy tabs distinguish full and filesystem configurations', async ({ page }) => {
  await page.goto('/performance/#regression-cases');
  const comparison = page.locator('[data-policy-switch]');
  const assuraTab = comparison.getByRole('tab', { name: 'Assura' });
  const lsLintTab = comparison.getByRole('tab', { name: 'LS-Lint' });
  await expect(assuraTab).toHaveAttribute('aria-selected', 'true');
  await expect(comparison.locator('.policy-wipe-layer.is-assura')).toBeVisible();
  await expect(comparison.locator('.policy-wipe-layer.is-lslint')).toBeHidden();
  await lsLintTab.click();
  await expect(lsLintTab).toHaveAttribute('aria-selected', 'true');
  await expect(comparison.locator('.policy-wipe-layer.is-lslint')).toBeVisible();
  await expect(comparison.locator('.policy-wipe-layer.is-assura')).toBeHidden();
});

test('performance policy renders the checked Assura YAML fixture exactly', async ({ page }) => {
  await page.goto('/performance/#regression-cases');
  const rendered = await page
    .locator('.policy-wipe-layer.is-assura .policy-code-line')
    .evaluateAll((lines) => lines.map((line) => {
      const indent = Number((line as HTMLElement).style.getPropertyValue('--indent'));
      return `${'  '.repeat(indent)}${line.textContent?.trim() ?? ''}`;
    }).join('\n'));
  expect(rendered).toBe(assuraPolicyFixture);
});

test('performance policy renders the checked LS-Lint fixture exactly', async ({ page }) => {
  await page.goto('/performance/#regression-cases');
  const rendered = await page
    .locator('.policy-wipe-layer.is-lslint .policy-code-line')
    .evaluateAll((lines) => lines.map((line) => {
      const indent = Number((line as HTMLElement).style.getPropertyValue('--indent'));
      return `${'  '.repeat(indent)}${line.textContent?.trim() ?? ''}`;
    }).join('\n'));
  expect(rendered).toBe(lsLintPolicyFixture);
});

test('performance policy tabs are touch-sized and support arrow-key switching', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/performance/#regression-cases');
  const comparison = page.locator('[data-policy-switch]');
  const assuraTab = comparison.getByRole('tab', { name: 'Assura' });
  const lsLintTab = comparison.getByRole('tab', { name: 'LS-Lint' });
  for (const tab of [assuraTab, lsLintTab]) {
    const box = await tab.boundingBox();
    expect(box?.height).toBeGreaterThanOrEqual(44);
  }
  await assuraTab.focus();
  await page.keyboard.press('ArrowRight');
  await expect(lsLintTab).toBeFocused();
  await expect(lsLintTab).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowLeft');
  await expect(assuraTab).toBeFocused();
  await expect(assuraTab).toHaveAttribute('aria-selected', 'true');
});

test('expanded performance cohort stays compact on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/performance/#benchmark-projects');
  await page.getByText('View all eight project measurements').click();
  const firstRow = page.locator('.benchmark-matrix-row:not(.benchmark-matrix-head)').first();
  await expect(firstRow).toBeVisible();
  const cells = await firstRow.locator('[role="cell"]').evaluateAll((items) =>
    items.map((item) => {
      const rect = item.getBoundingClientRect();
      return { x: Math.round(rect.x), y: Math.round(rect.y), width: Math.round(rect.width) };
    }),
  );
  expect(cells[0].width).toBe(cells[1].width);
  expect(cells[2].y).toBe(cells[3].y);
  expect(cells[2].x).toBeLessThan(cells[3].x);
  expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
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

test('review and check have distinct workflow roles without maturity badges', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const model = page.getByLabel('Review and check command roles');
  const review = model.getByRole('article').filter({ hasText: 'Review' });
  const check = model.getByRole('article').filter({ hasText: 'Check' });

  await expect(review).toContainText('During agent work');
  await expect(review).toContainText('Advisory signal');
  await expect(review).toContainText('Advisory result');
  await expect(check).toContainText('Before commit or merge');
  await expect(check).toContainText('Policy gate');
  await expect(check).toContainText('Pass / fail exit');
  await expect(page.locator('.journey-section')).not.toContainText('Experimental');
  expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
});

test('technical docs remain reachable', async ({ page }) => {
  const response = await page.goto('/guides/quickstart/');
  expect(response?.status()).toBe(200);
  await expect(page.locator('main')).toBeVisible();
});

for (const colorScheme of themes) {
  for (const width of [360, 390, 768, 1024, 1440]) {
    test(`${colorScheme} canonical docs at ${width}px match the product shell`, async ({ page }, testInfo) => {
      await page.emulateMedia({ colorScheme });
      await page.setViewportSize({ width, height: 900 });
      await page.goto('/reference/configuration/');
      await expect(page.getByRole('heading', { level: 1, name: 'Configuration Reference' })).toBeVisible();
      await expect(page.locator('header img').first()).toBeVisible();
      await expect(page.getByRole('heading', { level: 2, name: 'Concise Structure Notation' })).toBeVisible();
      expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
      const audit = await new AxeBuilder({ page })
        // Expressive Code renders each scrollable code block as an unlabeled region.
        .disableRules(['landmark-unique'])
        .analyze();
      expect(audit.violations).toEqual([]);
      await page.screenshot({
        path: testInfo.outputPath(`docs-${colorScheme}-${width}.png`),
        fullPage: true,
      });
    });
  }
}

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
