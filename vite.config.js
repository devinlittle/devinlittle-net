import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
//import basicSsl from '@vitejs/plugin-basic-ssl';
import fs from 'fs';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    https: {
      key: fs.readFileSync('./certs/localhostTRUE.pem'),
      cert: fs.readFileSync('./certs/localhostTRUE.crt'),
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },
  optimizeDeps: {
    exclude: ['@sqlite.org/sqlite-wasm']
  }
});
