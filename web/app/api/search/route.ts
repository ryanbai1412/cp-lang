import type { StructuredData } from 'fumadocs-core/mdx-plugins';
import { createFromSource } from 'fumadocs-core/search/server';
import { components } from 'zbsearch';
import { source } from '@/lib/source';

// The default word splitter keeps `SortedBag.last_true` whole, so a search for
// `last_true` missed every method; index each dotted name's parts as well.
const words = components.tokenizer.createTokenizer({ language: 'multilingual' });
const tokenizer = {
  language: words.language,
  normalizationCache: words.normalizationCache,
  tokenize(raw: string, language?: string, prop?: string, withCache?: boolean) {
    const tokens = words.tokenize(raw, language, prop, withCache);
    return [...tokens, ...tokens.flatMap((t) => (t.includes('.') ? t.split('.').filter(Boolean) : []))];
  },
};

type Page = ReturnType<typeof source.getPages>[number];

async function structured(page: Page): Promise<StructuredData> {
  const data = page.data as { structuredData?: StructuredData | (() => Promise<StructuredData>); load?: () => Promise<{ structuredData: StructuredData }> };
  if (typeof data.structuredData === 'function') return data.structuredData();
  if (data.structuredData) return data.structuredData;
  if (data.load) return (await data.load()).structuredData;
  throw new Error(`search: no structured data for ${page.url}`);
}

// Generated reference pages index their headings (one per item) and the
// page intro only: their full text made the index larger than the compiler.
// Core chapters also keep their table rows, where each method gets one line.
function brief(data: StructuredData, tables: boolean): StructuredData {
  return {
    headings: data.headings,
    contents: data.contents.flatMap((c) => {
      if (!c.heading) return [c];
      // A row is stored with its table's header; the row alone names the method.
      if (tables && (c as { table?: string }).table) return [{ ...c, content: c.content.split('\n').at(-1)! }];
      return [];
    }),
  };
}

export const revalidate = false;
export const { staticGET: GET } = createFromSource(source, {
  // Results rank by relevance; no query sorts by a field, so skip the sort index.
  sort: { enabled: false },
  tokenizer,
  async buildIndex(page) {
    const data = await structured(page);
    return {
      id: page.url,
      url: page.url,
      title: page.data.title ?? page.url,
      description: page.data.description,
      structuredData: page.url.startsWith('/docs/ref/') ? brief(data, page.url.startsWith('/docs/ref/core/')) : data,
    };
  },
});
