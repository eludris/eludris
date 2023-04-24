import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';

import { readFileSync, writeFileSync } from 'fs';
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
import { visit } from 'unist-util-visit';
import { toString } from 'mdast-util-to-string';

// we have to do this entire loop because code nodes are their own things so stuff like mdast-util-find-and-replace won't work
const remarkAutolinkReferenceEntries = () => {
  return (tree) => {
    const text = toMarkdown(tree, { extensions: [gfmTableToMarkdown()] });
    return fromMarkdown(
      text.replace(/\\\[`(.+)`\]/gm, (_, p1) => {
        const item = AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${p1}.json`));
        if (!item) {
          return p1;
        }
        return `[${p1
          .replace(/(?:^|_)([a-z0-9])/gm, (_, p1) => p1.toUpperCase())
          .replace(/[A-Z]/gm, ' $&')
          .trim()}](/reference/${item.split('.')[0]})`;
      }),
      { extensions: [gfmTable], mdastExtensions: [gfmTableFromMarkdown] }
    );
  };
};

const remarkGenerateSearchIndex = () => {
  return (tree, file) => {
    let sections = [{ line: 0, content: '' }];
    visit(tree, 'heading', (node) => {
      let text = toMarkdown(node).trim().replace(/^#+ /, '');
      let id = toString(node).trim().toLowerCase().replace(/ /gm, '-');
      sections.push({ line: node.position.start.line, text, id, content: '' });
    });
    sections = sections.sort((a, b) => b.line - a.line);
    let baseRoute;
    if (file.history[0]) {
      if (file.history[0].endsWith('/docs/index.md')) {
        baseRoute = '/';
      } else {
        baseRoute = `/${file.history[0].split('/').slice(-2).join('/').split('.')[0]}`;
      }
    } else {
      // probably an autodoc entry
      const entry = AUTODOC_ENTRIES.find(
        (e) =>
          sections[sections.length - 2].text ==
          e
            .split('/')[1]
            .split('.')[0]
            .replace(/(?:^|_)([a-z0-9])/gm, (_, p1) => p1.toUpperCase())
            .replace(/[A-Z]/gm, ' $&')
            .trim()
      ).split('.')[0];
      baseRoute = `/reference/${entry}`;
    }
    visit(tree, 'paragraph', (node) => {
      for (let i = 0; i < sections.length; i++) {
        if (sections[i].line < node.position.start.line) {
          sections[i].content += toString(node).replace(/\n/gm, ' ');
          return;
        }
      }
      sections[sections.length - 1].content += toString(node).replace(/\n/gm, ' ');
    });
    visit(tree, 'tableCell', (node) => {
      for (let i = 0; i < sections.length; i++) {
        if (sections[i].line < node.position.start.line) {
          sections[i].content += ' ' + toString(node).replace(/\n/gm, ' ');
          return;
        }
      }
      sections[sections.length - 1].content += ' ' + toString(node).replace(/\n/gm, ' ');
    });
    sections = sections
      .filter((s) => s.text)
      .map((s) => ({ route: `${baseRoute}${s.id ? '#' + s.id : ''} `, ...s }));
    const data = JSON.parse(readFileSync('public/search.json'));
    writeFileSync('public/search.json', JSON.stringify([...data, ...sections.reverse()]));
  };
};

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
      remarkGenerateSearchIndex
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
