import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './src/tests',
  outputDir: './test-results',
  fullyParallel: true,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],
  use: {
    baseURL: 'http://127.0.0.1:4322',
    browserName: 'chromium',
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'pnpm dev --host 127.0.0.1 --port 4322',
    url: 'http://127.0.0.1:4322/',
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
