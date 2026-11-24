import { defineConfig } from 'vitest/config';

// handbook/guidelines/site-checklist.md site/R20: unit coverage of site code never falls.
export default defineConfig({
  test: {
    coverage: {
      enabled: true,
      provider: 'v8',
      include: ['lib/**/*.{ts,tsx}', 'components/**/*.{ts,tsx}', 'scripts/**/*.{mjs,ts}'],
      reporter: ['text-summary'],
      thresholds: { statements: 14.5, branches: 13.8, functions: 12.1, lines: 14.5 },
    },
  },
});
