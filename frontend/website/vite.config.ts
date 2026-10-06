import adapter from '@sveltejs/adapter-bun';
import { enhancedImages } from "@sveltejs/enhanced-img";
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import fs from 'fs';

export default defineConfig({
  plugins: [
    sveltekit({
      compilerOptions: { warningFilter: (warning) => !warning.code.startsWith('a11y') },
      extensions: ['.svelte'] /* '.md', ".svx"], */,
      adapter: adapter(
        {
          buildOptions: {
            compile: {
              target: (process.env.TARGET ?? 'bun-linux-arm64') as Bun.Build.CompileTarget,
              outfile: 'devinlittle-net',
            },
            minify: true,
            bytecode: true,
            sourcemap: 'linked'
          }
        },
      )
    }),
    enhancedImages(),
  ],
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
