import adapter from '@sveltejs/adapter-node';
import { enhancedImages } from "@sveltejs/enhanced-img"
import { mdsvex } from 'mdsvex';

export default {
  kit: {
    adapter: adapter({
      out: 'build',
      precompress: true,
      envPrefix: 'PRODUCTION_',
      polyfill: true,
      //preprocess: preprocess({sass: true}),
    }),
  },
  extensions: ['.svelte', '.md', ".svx"],
  preprocess: [
    enhancedImages(),
    mdsvex({
      extensions: ['.md', '.svx']
    })
  ]
};
