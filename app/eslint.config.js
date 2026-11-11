import js from '@eslint/js';
import reactHooks from 'eslint-plugin-react-hooks';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist', 'src/wasm-pkg', 'test-results', 'playwright-report'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  reactHooks.configs.flat.recommended,
  // Plain Node scripts that also run code inside a browser page.
  {
    files: ['e2e/**/*.mjs'],
    languageOptions: {
      globals: { process: 'readonly', console: 'readonly', performance: 'readonly', requestAnimationFrame: 'readonly' },
    },
  },
);
