// Throwaway config for the Phase 1 spike pages. Never used by the app build.
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  root: import.meta.dirname,
  base: './',
  plugins: [react()],
  server: { fs: { allow: ['..'] } },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2022',
    rollupOptions: { input: { trace: `${import.meta.dirname}/trace/index.html` } },
  },
});
