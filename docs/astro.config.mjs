import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';

import sitemap from '@astrojs/sitemap';
import { rehypeAccessibleEmojis } from 'rehype-accessible-emojis';
import rehypeAutolinkHeadings from 'rehype-autolink-headings/lib';
import { h } from 'hastscript';
import rehypeSlug from 'rehype-slug';
import AUTODOC_ENTRIES from './public/autodoc/index.json';
import { gfmTableFromMarkdown, gfmTableToMarkdown } from 'mdast-util-gfm-table';
import { toMarkdown } from 'mdast-util-to-markdown';
import { fromMarkdown } from 'mdast-util-from-markdown';
import { gfmTable } from 'micromark-extension-gfm-table';

// we have to do this entire loop because code nodes are their own things so stuff like mdast-util-find-and-replace won't work
const remarkAutolinkReferenceEntries = () => {
  return (tree) => {
    const text = toMarkdown(tree, { extensions: [gfmTableToMarkdown()] });
    return fromMarkdown(text.replace(/\\\[`(.+)`\]/gm, (_, p1) => {
      const item =
        AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${p1}.json`));
      if (!item) {

      }
      return `[${p1.replace(/(?:^|_)([a-z0-9])/gm, (_, p1) => p1.toUpperCase()).replace(/[A-Z]/gm, ' $&')}](/reference/${item.split('.')[0]})`;
    }), { extensions: [gfmTable], mdastExtensions: [gfmTableFromMarkdown] });
  };
}

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
    remarkPlugins: [
      remarkAutolinkReferenceEntries,
    ],
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
