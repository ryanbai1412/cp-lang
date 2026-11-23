import { defineConfig } from 'vitest/config';

// handbook/guidelines/site-checklist.md site/R20: unit coverage of site code never falls.
export default defineConfig({
  test: {
    coverage: {
      enabled: true,
      provider: 'v8',
      include: ['lib/**/*.{ts,tsx}', 'components/**/*.{ts,tsx}', 'scripts/**/*.{mjs,ts}'],
      reporter: ['text-summary'],
      thresholds: { statements: 7.8, branches: 9.5, functions: 5.2, lines: 7.6 },
    },
  },
});
