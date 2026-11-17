// SPDX-License-Identifier: AGPL-3.0-or-later
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const design = fileURLToPath(new URL('../../design/system', import.meta.url));
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  resolve: { alias: { '@design': design, $lib: fileURLToPath(new URL('./src/lib', import.meta.url)) } },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    fs: { allow: ['.', design] },
    watch: { ignored: ['**/src-tauri/**'] },
  },
  build: { target: 'safari16', sourcemap: false, chunkSizeWarningLimit: 800 },
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
