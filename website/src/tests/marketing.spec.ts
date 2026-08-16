import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'node:fs';
import {
  agentSetupPrompt,
  installCommand,
  sourcePreviewRevision,
} from '../data/marketing';

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

type DemoLine = {
  text: string;
  tone: 'prompt' | 'plain' | 'muted' | 'info' | 'warn' | 'pass' | 'fail';
};

const reviewDemo = JSON.parse(
  readFileSync(new URL('../data/review-demo.json', import.meta.url), 'utf8'),
) as { command: string; hero_lines: DemoLine[]; display_lines: DemoLine[]; artifact_lines: DemoLine[] };

const checkDemo = JSON.parse(
  readFileSync(new URL('../data/check-demo.json', import.meta.url), 'utf8'),
) as { command: string; artifact_lines: DemoLine[] };

const onboardingDemo = JSON.parse(
  readFileSync(new URL('../data/onboarding-demo.json', import.meta.url), 'utf8'),
) as {
  command: string;
  artifact_lines: DemoLine[];
  integration: { generated: boolean; activated: boolean; verified: boolean; conflicted: boolean };
};

const intelligenceDemo = JSON.parse(
  readFileSync(new URL('../data/intelligence-demo.json', import.meta.url), 'utf8'),
) as { command: string; artifact_lines: DemoLine[] };

const policyDemo = JSON.parse(
  readFileSync(new URL('../data/policy-demo.json', import.meta.url), 'utf8'),
) as {
  config: string;
  tree: Array<{ path: string; full_path?: string; status: string; detail: string }>;
};

const performanceReport = JSON.parse(
  readFileSync(new URL('../../public/data/performance/current.json', import.meta.url), 'utf8'),
) as {
  ls_lint_package: string;
  results: Array<{
    fixture_id: string;
    fixture_cohort: string;
    row_family: string;
    median_runtime_ms: number;
    checked_file_count: number;
    directory_count: number;
    rule_count: number;
    native_ls_lint_parity: boolean;
    shared_config_id: string;
    status: string;
  }>;
};

const formatMeasuredMs = (value: number) => `${value.toFixed(value >= 10 ? 1 : 2)} ms`;
const formatCount = (value: number) => value.toLocaleString('en-US');

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

  const dialog = page.getByRole('dialog', { name: 'Set up this project with Assura.' });
  await expect(dialog).toBeVisible();
  await expect(page.getByRole('button', { name: 'Close setup dialog' })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

test('header setup entry stays visually secondary to the hero action', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/');

  const start = page.locator('.nav-start');
  const primaryAction = page.locator('.hero-actions .button-primary');
  const [startStyle, primaryStyle] = await Promise.all([
    start.evaluate((element) => {
      const style = getComputedStyle(element);
      return { background: style.backgroundColor, color: style.color };
    }),
    primaryAction.evaluate((element) => {
      const style = getComputedStyle(element);
      return { background: style.backgroundColor, color: style.color };
    }),
  ]);

  expect(startStyle.background).not.toBe(primaryStyle.background);
  expect(startStyle.color).not.toBe(primaryStyle.color);
});

for (const colorScheme of themes) {
  for (const width of [320, 360, 390]) {
    test(
      `${colorScheme} setup at ${width}px keeps onboarding simple and installation inspectable`,
      async ({ page }, testInfo) => {
        await page.emulateMedia({ colorScheme });
        await page.setViewportSize({ width, height: width < 390 ? 640 : 844 });
        await page.goto('/');
        await page.getByRole('link', { name: /with your agent/ }).first().click();

        const dialog = page.getByRole('dialog', { name: 'Set up this project with Assura.' });
        await expect(
          dialog.getByText('assura agent onboard .', { exact: true }),
        ).toBeVisible();
        await expect(dialog.locator('#agent-instruction')).toContainText(
          'Define project-owned rules for the expected stack',
        );
        const primaryCopy = dialog.getByRole('button', { name: 'Copy setup instruction' });
        const primaryCopyBox = await primaryCopy.boundingBox();
        expect(primaryCopyBox?.y).toBeGreaterThanOrEqual(0);
        expect((primaryCopyBox?.y ?? 0) + (primaryCopyBox?.height ?? 0)).toBeLessThanOrEqual(
          width < 390 ? 640 : 844,
        );
        const installation = dialog.getByRole('group', {
          name: `Installation details for revision ${sourcePreviewRevision.slice(0, 7)}`,
        });
        await expect(installation).not.toHaveAttribute('open', '');
        await dialog.screenshot({
          path: testInfo.outputPath(`setup-dialog-${colorScheme}-${width}.png`),
        });
        await installation.locator('summary').click();
        await expect(dialog.locator('#install-command')).toHaveText(installCommand);
        await expect(dialog.locator('#agent-prompt')).toHaveText(agentSetupPrompt);
        await expect(dialog.locator('#agent-prompt')).toContainText(sourcePreviewRevision);
        await expect(dialog).not.toContainText('assura.dev/install.sh');
        expect(await dialog.evaluate((element) => element.scrollWidth - element.clientWidth)).toBe(0);
        expect(
          await dialog.locator('#install-command').evaluate(
            (element) => element.scrollWidth - element.clientWidth,
          ),
        ).toBe(0);
        for (const control of [
          dialog.getByRole('button', { name: 'Close setup dialog' }),
          primaryCopy,
          dialog.getByRole('button', { name: 'Copy', exact: true }),
        ]) {
          const box = await control.boundingBox();
          expect(box?.height).toBeGreaterThanOrEqual(44);
        }
      },
    );
  }
}

test('setup copies the complete pinned evidence-first instruction', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: {
        writeText: (value: string) => {
          (window as Window & { copiedSetup?: string }).copiedSetup = value;
          return Promise.resolve();
        },
      },
    });
  });
  await page.goto('/');
  await page.getByRole('link', { name: /with your agent/ }).first().click();
  await page.getByRole('button', { name: 'Copy setup instruction' }).click();
  expect(
    await page.evaluate(() => (window as Window & { copiedSetup?: string }).copiedSetup),
  ).toBe(agentSetupPrompt);
});

test('setup reports clipboard failures without hiding the fallback', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: () => Promise.reject(new Error('clipboard unavailable')) },
    });
  });
  await page.goto('/');
  await page.getByRole('link', { name: /with your agent/ }).first().click();
  const dialog = page.getByRole('dialog', { name: 'Set up this project with Assura.' });
  await dialog.getByRole('button', { name: 'Copy setup instruction' }).click();
  await expect(dialog.getByRole('status')).toHaveText(
    'Copy failed. The setup instruction is selected for manual copy.',
  );
  await expect(dialog.locator('#agent-prompt')).toBeVisible();
  await expect(dialog.locator('#agent-prompt')).toBeFocused();
});

test('short mobile view keeps the project review visible', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 640 });
  await page.goto('/');
  const review = page.locator('[data-terminal-variant="review"]').first();
  await expect(review.locator('.terminal-bar strong')).toHaveText(reviewDemo.command);
  await expect(review.locator('[data-terminal-line]').first()).toBeVisible();
});

test('homepage output is an exact selection of the supported review renderer', async ({ page }) => {
  await page.goto('/');
  const terminal = page.locator('[data-terminal-variant="review"]').first();
  await expect(terminal.locator('.terminal-bar strong')).toHaveText(reviewDemo.command);
  expect(await terminal.locator('[data-terminal-line]').allTextContents()).toEqual(
    reviewDemo.hero_lines.map((line) => line.text),
  );
  expect(await terminal.locator('[data-terminal-line]').evaluateAll((lines) =>
    lines.map((line) => [...line.classList].find((name) => name !== 'terminal-line')),
  )).toEqual(reviewDemo.hero_lines.map((line) => line.tone));
});

test('terminal output uses ANSI-like text emphasis without per-line panels', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/');
  const lines = page.locator('[data-terminal-variant="review"] [data-terminal-line]');
  await expect(lines.first()).toBeVisible();
  const styles = await lines.evaluateAll((items) => items.map((item) => {
    const style = getComputedStyle(item);
    return {
      background: style.backgroundColor,
      boxShadow: style.boxShadow,
      paddingLeft: Number.parseFloat(style.paddingLeft),
      color: style.color,
    };
  }));

  expect(styles.every((style) => style.background === 'rgba(0, 0, 0, 0)')).toBe(true);
  expect(styles.every((style) => style.boxShadow === 'none')).toBe(true);
  const failureColor = await page.locator('.terminal-token-status.is-danger').first().evaluate(
    (item) => getComputedStyle(item).color,
  );
  const commandColor = await page.locator('.terminal-token-command').first().evaluate(
    (item) => getComputedStyle(item).color,
  );
  const titleColor = await page.locator('.terminal-token-title').first().evaluate(
    (item) => getComputedStyle(item).color,
  );
  const labelColor = await page.locator('.terminal-token-label').first().evaluate(
    (item) => getComputedStyle(item).color,
  );
  expect(failureColor).not.toBe(commandColor);
  expect(titleColor).not.toBe(commandColor);
  expect(labelColor).not.toBe(commandColor);

  await page.goto('/project-review/');
  const infoColor = await page.locator('.terminal-line.info').first().evaluate(
    (item) => getComputedStyle(item).color,
  );
  expect(infoColor).not.toBe(commandColor);
});

test('review output separates row labels, metric names, and summarized values', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/');

  const terminal = page.locator('[data-terminal-variant="review"]').first();
  const watch = terminal.locator('[data-terminal-line]').filter({ hasText: /^Watch/ });
  const rowLabel = watch.locator('[data-terminal-label]');
  const metricKey = watch.locator('[data-terminal-metric-key]').first();
  const metricValue = watch.locator('[data-terminal-metric-value]').first();

  await expect(rowLabel).toHaveText('Watch');
  await expect(metricKey).toHaveText('blocking-validation=');
  await expect(metricValue).toHaveText('1/1');

  const hierarchy = await Promise.all([rowLabel, metricKey, metricValue].map((item) =>
    item.evaluate((element) => {
      const style = getComputedStyle(element);
      return { color: style.color, weight: Number.parseInt(style.fontWeight, 10) };
    }),
  ));
  expect(new Set(hierarchy.map((item) => item.color)).size).toBe(3);
  expect(hierarchy[2].weight).toBeGreaterThan(hierarchy[1].weight);

  const crossedThreshold = watch.locator('[data-terminal-metric-value="danger"]');
  await expect(crossedThreshold).toHaveCount(1);
  await expect(crossedThreshold).toHaveText('1/1');
});

test('review rows use hanging indents and semantic section breaks', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/');

  const terminal = page.locator('[data-terminal-variant="review"]').first();
  const branch = terminal.locator('[data-terminal-line]').filter({ hasText: /^Branch/ });
  const summary = terminal.locator('[data-terminal-line]').first();
  const findings = terminal.locator('[data-terminal-line]').filter({ hasText: /^Findings/ });
  const fixFirst = terminal.locator('[data-terminal-line]').filter({ hasText: /^Fix first/ });

  await expect(branch).toHaveAttribute('data-terminal-labelled', 'true');
  await expect(findings).toHaveAttribute('data-terminal-section-start', 'true');
  await expect(summary).toHaveAttribute('data-terminal-summary', 'true');
  await expect(fixFirst).toHaveAttribute('data-terminal-section-start', 'true');

  const layout = await branch.evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      paddingInlineStart: Number.parseFloat(style.paddingInlineStart),
      textIndent: Number.parseFloat(style.textIndent),
    };
  });
  expect(layout.paddingInlineStart).toBeGreaterThan(0);
  expect(layout.textIndent).toBeLessThan(0);
  const fontSize = await terminal.locator('pre').evaluate(
    (element) => Number.parseFloat(getComputedStyle(element).fontSize),
  );
  expect(fontSize).toBeGreaterThanOrEqual(12);
});

test('review output reserves warning color for actual warnings', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/');

  const terminal = page.locator('[data-terminal-variant="review"]').first();
  for (const label of ['Branch', 'Worktree', 'Hot path']) {
    const row = terminal.locator('[data-terminal-line]').filter({ hasText: new RegExp(`^${label}`) });
    await expect(row.locator('[data-terminal-metric-value="warning"]')).toHaveCount(0);
  }

  await expect(terminal.locator('.terminal-token-status.is-warning')).toHaveText('needs attention');
  await expect(terminal.locator('.terminal-token-status.is-danger')).toContainText('BadName.tsx');
});

test('example output CTA connects project policy to pass and fail paths', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await page.getByRole('link', { name: 'See how rules apply' }).click();
  await expect(page.getByRole('heading', { name: '.assura/config.yml' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Policy, paths, and violations' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: 'agent-doc:' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: './**/:' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: '.ts: kebab-case | max_lines:500' })).toBeVisible();
  await expect(page.locator('.config-line').filter({ hasText: 'AGENTS.md: exists:1 | $agent-doc' }).first()).toBeVisible();
  const configMarkers = await page.locator('.policy-panel .rule-marker').allTextContents();
  const treeMarkers = await page.locator('.tree-panel .rule-marker').allTextContents();
  expect(new Set(configMarkers)).toEqual(new Set(treeMarkers));
  await expect(page.getByLabel('Observed fixture path').first()).toBeVisible();
  await expect(page.getByLabel('Check violation').first()).toBeVisible();
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
    expect(policyDemo.config).toBe(homepagePolicyFixture);
    expect(rendered.join('\n')).toBe(policyDemo.config);
  });
}

test('policy tree paths, states, and measured values come from the executable fixture', async ({ page }) => {
  await page.goto('/#review-output');
  const rows = await page.locator('[data-policy-path]').evaluateAll((items) =>
    items.map((item) => ({
      path: (item as HTMLElement).dataset.policyPath,
      status: (item as HTMLElement).dataset.policyStatus,
      detail: (item as HTMLElement).dataset.policyDetail,
    })),
  );
  expect(rows).toEqual(policyDemo.tree.map((row) => ({
    path: row.full_path ?? row.path,
    status: row.status,
    detail: row.detail,
  })));
  expect(rows).toContainEqual({
    path: 'packages/core/src/user-menu.ts',
    status: 'observed',
    detail: '184 / 500 lines in passing fixture',
  });
  expect(rows).toContainEqual({
    path: 'packages/core/src/checkout-flow.ts',
    status: 'violation',
    detail: '537 / 500 lines',
  });
});

test('policy tree keeps file paths readable before mobile details wrap', async ({ page }) => {
  for (const width of [320, 360, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await page.goto('/#review-output');
    const row = page.locator('[data-policy-path="packages/core/src/user-menu.ts"]');
    const layout = await row.evaluate((item) => {
      const path = item.querySelector('code')!;
      const detail = item.querySelector('small')!;
      const pathRect = path.getBoundingClientRect();
      const detailRect = detail.getBoundingClientRect();
      const style = getComputedStyle(path);
      const fontSize = Number.parseFloat(style.fontSize);
      const lineHeight = Number.parseFloat(style.lineHeight);
      return {
        path: path.textContent,
        whiteSpace: style.whiteSpace,
        pathHeight: Math.round(pathRect.height),
        lineHeight: Number.isFinite(lineHeight) ? lineHeight : fontSize * 1.4,
        detailTop: Math.round(detailRect.top),
        pathBottom: Math.round(pathRect.bottom),
      };
    });

    expect(layout.path).toBe('user-menu.ts');
    expect(layout.whiteSpace).toBe('nowrap');
    expect(layout.pathHeight).toBeLessThanOrEqual(Math.ceil(layout.lineHeight * 1.25));
    expect(layout.detailTop).toBeGreaterThanOrEqual(layout.pathBottom);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
  }
});

const rendererPages = [
  { path: '/project-review/', variant: 'review', title: reviewDemo.command, lines: reviewDemo.artifact_lines },
  { path: '/ai-coding-agent-guardrails/', variant: 'review', title: reviewDemo.command, lines: reviewDemo.artifact_lines },
  { path: '/repository-validation/', variant: 'check', title: checkDemo.command, lines: checkDemo.artifact_lines },
  { path: '/agent-onboarding/', variant: 'onboarding', title: onboardingDemo.command, lines: onboardingDemo.artifact_lines },
  { path: '/project-intelligence/', variant: 'neutral', title: intelligenceDemo.command, lines: intelligenceDemo.artifact_lines },
] as const;

for (const rendererPage of rendererPages) {
  test(`${rendererPage.path} renders the supported CLI artifact exactly`, async ({ page }) => {
    await page.goto(rendererPage.path);
    const terminal = page.locator(`[data-terminal-variant="${rendererPage.variant}"]`).first();
    await expect(terminal.locator('.terminal-bar strong')).toHaveText(rendererPage.title);
    expect(await terminal.locator('[data-terminal-line]').allTextContents()).toEqual(
      rendererPage.lines.map((line) => line.text),
    );
    expect(await terminal.locator('[data-terminal-line]').evaluateAll((lines) =>
      lines.map((line) => [...line.classList].find((name) => name !== 'terminal-line')),
    )).toEqual(rendererPage.lines.map((line) => line.tone));
  });
}

for (const colorScheme of themes) {
  for (const width of widths) {
    test(`${colorScheme} renderer artifacts at ${width}px wrap without changing product state`, async ({ page }, testInfo) => {
      await page.emulateMedia({ colorScheme });
      await page.setViewportSize({ width, height: 900 });
      for (const rendererPage of rendererPages) {
        await page.goto(rendererPage.path);
        const terminal = page.locator(`[data-terminal-variant="${rendererPage.variant}"]`).first();
        await expect(terminal).toBeVisible();
        expect(await terminal.locator('pre').evaluate((element) => element.scrollWidth - element.clientWidth)).toBe(0);
        expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
        await terminal.screenshot({
          path: testInfo.outputPath(`${rendererPage.variant}-${rendererPage.path.split('/').filter(Boolean).join('-')}-${colorScheme}-${width}.png`),
        });
      }
    });
  }
}

test('review, check, and onboarding retain distinct real product states', async ({ page }) => {
  await page.goto('/project-review/');
  const review = page.locator('[data-terminal-variant="review"]').first();
  await expect(review.locator('[data-terminal-line]').filter({ hasText: /^Status/ })).toContainText('needs attention');
  await expect(review.locator('[data-terminal-line]').filter({ hasText: /^Fix first/ })).toContainText('BadName.tsx');

  await page.goto('/repository-validation/');
  const check = page.locator('[data-terminal-variant="check"]').first();
  await expect(check.locator('[data-terminal-line]').filter({ hasText: 'Blocking: true' })).toBeVisible();
  await expect(check.locator('[data-terminal-line]').filter({ hasText: 'Fix:' })).toBeVisible();

  await page.goto('/agent-onboarding/');
  const onboarding = page.locator('[data-terminal-variant="onboarding"]').first();
  expect(onboardingDemo.integration).toMatchObject({
    generated: true,
    activated: true,
    verified: true,
    conflicted: false,
  });
  await expect(onboarding.locator('[data-terminal-line]').filter({ hasText: /^Host/ })).toContainText(
    'generated=true activated=true verified=true conflicted=false',
  );
});

test('performance CTA lands on the measured project cohort', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('link', { name: 'How we measured it' }).click();
  await expect(page).toHaveURL(/\/performance\/#measured-comparison$/);
  await expect(page.getByRole('heading', { name: 'Faster than native LS-Lint in all eight cold comparisons.' })).toBeVisible();
  await expect(page.locator('.policy-breadth-card')).toHaveCount(1);
  await expect(page.locator('.policy-wipe-layer')).toHaveCount(2);
  await expect(page.getByRole('slider')).toHaveCount(0);
  await expect(page.getByText('The extra lines define quality policies LS-Lint cannot express.', { exact: true })).toBeVisible();
  await expect(page.locator('.policy-coverage-row')).toHaveCount(8);
  await expect(page.getByText('Quality policy example', { exact: true })).toBeVisible();
  await expect(page.locator('.benchmark-card')).toHaveCount(3);
  await expect(page.locator('.benchmark-config')).toHaveCount(3);
  await expect(page.locator('.benchmark-matrix-row:not(.benchmark-matrix-head)')).toHaveCount(8);
  await expect(page.locator('.benchmark-matrix-row').filter({ hasText: 'Multipart extension scale' })).toContainText('1,501 checked');
  await expect(page.locator('.benchmark-matrix-row').filter({ hasText: 'Configured scope scale' })).toContainText('801 rules');
  await expect(page.getByText('Download current JSON')).toHaveCount(0);
});

test('performance ranges use labeled report-derived minimum and maximum values', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/performance/#regression-cases');
  const displayedFixtureIds = new Set(
    await page.locator('.benchmark-matrix-row:not(.benchmark-matrix-head)').evaluateAll((rows) =>
      rows.map((row) => (row as HTMLElement).dataset.fixtureId),
    ),
  );
  const coldRows = performanceReport.results.filter(
    (result) => result.status === 'pass'
      && result.row_family === 'assura-cli'
      && displayedFixtureIds.has(result.fixture_id),
  );
  const expectedRange = (field: 'checked_file_count' | 'directory_count' | 'rule_count') => {
    const values = coldRows.map((row) => row[field]);
    return `${formatCount(Math.min(...values))} to ${formatCount(Math.max(...values))}`;
  };

  const range = page.locator('.benchmark-range');
  await expect(range.locator('[data-range="files"]')).toContainText('Checked files');
  await expect(range.locator('[data-range="files"] strong')).toHaveText(expectedRange('checked_file_count'));
  await expect(range.locator('[data-range="directories"] strong')).toHaveText(expectedRange('directory_count'));
  await expect(range.locator('[data-range="rules"] strong')).toHaveText(expectedRange('rule_count'));
  expect(await range.innerText()).not.toMatch(/\d-\d/);
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
  const assuraTab = comparison.getByRole('tab', { name: /^Assura/ });
  const lsLintTab = comparison.getByRole('tab', { name: /^LS-Lint/ });
  await expect(assuraTab).toHaveAttribute('aria-selected', 'true');
  await expect(comparison.locator('.policy-wipe-layer.is-assura')).toBeVisible();
  await expect(comparison.locator('.policy-wipe-layer.is-lslint')).toBeHidden();
  await lsLintTab.click();
  await expect(lsLintTab).toHaveAttribute('aria-selected', 'true');
  await expect(comparison.locator('.policy-wipe-layer.is-lslint')).toBeVisible();
  await expect(comparison.locator('.policy-wipe-layer.is-assura')).toBeHidden();
  await expect(comparison.locator('.policy-wipe-actions + .policy-wipe-stage')).toBeVisible();
  await expect(comparison.locator('.policy-coverage-note')).toContainText(
    'All 6 red limitations are exercised against the same pinned native LS-Lint 2.3.0 package',
  );
  await expect(comparison.locator('.policy-code-line.is-gap')).toHaveCount(6);
  await expect(comparison.locator('.policy-code-line.is-gap')).toContainText([
    'SKILL.md content and file line count remain unchecked',
    'SKILL.md is named when present, not required by this absent-safe config',
    'optional trees cannot require nested files without failing when absent',
    'optional package peers must be ignored to preserve exact required counts',
    'no aggregate child total across mixed files and directories',
    'no file line limit, severity, or rule-specific repair message',
  ]);
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
  const assuraTab = comparison.getByRole('tab', { name: /^Assura/ });
  const lsLintTab = comparison.getByRole('tab', { name: /^LS-Lint/ });
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

test('performance cohort compares all eight variables and timings on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/performance/#benchmark-projects');
  const rows = page.locator('.benchmark-matrix-row:not(.benchmark-matrix-head)');
  await expect(rows).toHaveCount(8);
  for (const row of await rows.all()) {
    await expect(row).not.toContainText('n/a');
    await expect(row).toContainText('Assura');
    await expect(row).toContainText('LS-Lint');
    await expect(row).toContainText('faster cold');
    await expect(row).toContainText('warm');
    const fixtureId = await row.getAttribute('data-fixture-id');
    const fixtureRows = performanceReport.results.filter(
      (result) => result.fixture_id === fixtureId && result.status === 'pass',
    );
    const assura = fixtureRows.find((result) => result.row_family === 'assura-cli');
    const lsLint = fixtureRows.find((result) => result.row_family === 'ls-lint-cli');
    const warm = fixtureRows.find(
      (result) => result.row_family === 'assura-check-dirty-project-session-cli',
    );
    expect(assura).toBeDefined();
    expect(lsLint).toBeDefined();
    expect(warm).toBeDefined();
    await expect(row).toHaveAttribute('data-fixture-cohort', assura!.fixture_cohort);
    await expect(row).toHaveAttribute('data-shared-config-id', assura!.shared_config_id);
    await expect(row).toHaveAttribute('data-native-parity', 'true');
    expect(lsLint!.shared_config_id).toBe(assura!.shared_config_id);
    expect(lsLint!.native_ls_lint_parity).toBe(true);
    await expect(row).toContainText(`Assura ${formatMeasuredMs(assura!.median_runtime_ms)}`);
    await expect(row).toContainText(`LS-Lint ${formatMeasuredMs(lsLint!.median_runtime_ms)}`);
    await expect(row).toContainText(`${formatMeasuredMs(warm!.median_runtime_ms)} warm`);
    await expect(row).toContainText(
      `${(lsLint!.median_runtime_ms / assura!.median_runtime_ms).toFixed(2)}x faster cold`,
    );
    await expect(row).toContainText(
      `${(lsLint!.median_runtime_ms / warm!.median_runtime_ms).toFixed(1)}x faster`,
    );
  }
  const firstRow = rows.first();
  await expect(firstRow).toContainText('Small startup-sensitive tree');
  await expect(firstRow).toContainText('Assura');
  await expect(firstRow).toContainText('LS-Lint');
  await expect(firstRow).toContainText('faster cold');

  for (const colorScheme of themes) {
    await page.emulateMedia({ colorScheme });
    for (const width of [320, 390, 768]) {
      await page.setViewportSize({ width, height: 844 });
      await expect(firstRow).toBeVisible();
      const cells = await firstRow.locator('[role="cell"]').evaluateAll((items) =>
        items.map((item) => {
          const rect = item.getBoundingClientRect();
          return {
            x: Math.round(rect.x),
            y: Math.round(rect.y),
            width: Math.round(rect.width),
          };
        }),
      );
      expect(cells[0].width).toBe(cells[2].width);
      expect(cells[1].y).toBe(cells[3].y);
      expect(cells[1].x).toBeLessThan(cells[3].x);
      expect(cells[2].y).toBeGreaterThan(cells[1].y);
      const visualHierarchy = await firstRow.evaluate((row) => {
        const focus = row.querySelector<HTMLElement>('.benchmark-focus-cell')!;
        const speed = row.querySelector<HTMLElement>('.benchmark-speed-cell')!;
        const ratio = speed.querySelector<HTMLElement>('strong span')!;
        return {
          focusBackground: getComputedStyle(focus).backgroundColor,
          speedBackground: getComputedStyle(speed).backgroundColor,
          ratioColor: getComputedStyle(ratio).color,
          bodyColor: getComputedStyle(row).color,
          ratioSize: Number.parseFloat(getComputedStyle(ratio).fontSize),
        };
      });
      expect(visualHierarchy.speedBackground).not.toBe(visualHierarchy.focusBackground);
      expect(visualHierarchy.ratioColor).not.toBe(visualHierarchy.bodyColor);
      expect(visualHierarchy.ratioSize).toBeGreaterThanOrEqual(20);
      expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBe(0);
      const rowOverflows = await rows.evaluateAll((items) =>
        items.map((item) => item.scrollWidth - item.clientWidth),
      );
      expect(rowOverflows).toEqual(Array(8).fill(0));
    }
  }
});

test('benchmark cohort anchor clears the sticky header', async ({ page }) => {
  for (const width of [390, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto('/performance/');
    await page.getByRole('link', { name: 'Compare all eight cases' }).click();
    await expect(page).toHaveURL(/#benchmark-projects$/);
    await expect.poll(async () => {
      const header = await page.locator('.site-header').boundingBox();
      const cohort = await page.locator('#benchmark-projects').boundingBox();
      const gap = (cohort?.y ?? 0) - ((header?.y ?? 0) + (header?.height ?? 0));
      return gap >= 8 && gap <= 48;
    }, { timeout: 15_000 }).toBe(true);
  }
});

test('diagnostic fixture internals stay collapsed until requested', async ({ page }) => {
  await page.goto('/performance/#benchmark-projects');
  const fixture = page.locator('details.benchmark-card').first();
  await expect(fixture).not.toHaveAttribute('open', '');
  await expect(fixture.locator('.benchmark-config')).toBeHidden();
  await fixture.locator('summary').click();
  await expect(fixture).toHaveAttribute('open', '');
  await expect(fixture.locator('.benchmark-config')).toBeVisible();
});

test('setup actions retain a no-JavaScript onboarding destination', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('link', { name: 'Start with your agent' }).first()).toHaveAttribute('href', '#onboard');
});

test('landing and setup dialog pass automated accessibility checks', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  const landing = await new AxeBuilder({ page }).analyze();
  expect(landing.violations).toEqual([]);

  await page.getByRole('link', { name: 'Start with your agent' }).first().click();
  const dialog = page.getByRole('dialog', { name: 'Set up this project with Assura.' });
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
  for (const width of [360, 390, 430, 768, 1024, 1440]) {
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
  test.setTimeout(90_000);
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
