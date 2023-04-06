import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';

import sitemap from '@astrojs/sitemap';
import { rehypeAccessibleEmojis } from 'rehype-accessible-emojis';
import rehypeAutolinkHeadings from 'rehype-autolink-headings/lib';
import { h } from 'hastscript';
import rehypeSlug from 'rehype-slug';

// https://astro.build/config
export default defineConfig({
  site: 'https://elusite.pages.dev',
  integrations: [mdx(), sitemap()],
  vite: {
    ssr: {
      external: ['svgo']
    }
  },
  markdown: {
    syntaxHighlight: 'prism',
    rehypePlugins: [
      rehypeAccessibleEmojis,
      rehypeSlug,
      [
        rehypeAutolinkHeadings,
        {
          behavior: 'before',
          content() {
            return h('span.header-icon', '>>');
          },
          group() {
            return h('span.header');
          }
        }
      ]
    ]
  }
});
