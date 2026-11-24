import type { StructuredData } from 'fumadocs-core/mdx-plugins';
import { createFromSource } from 'fumadocs-core/search/server';
import { components } from 'zbsearch';
import { source } from '@/lib/source';

// The default word splitter keeps `SortedBag.last_true` whole, so a search for
// `last_true` missed every method; index each dotted name's last part as well.
// The owner's own heading already finds `SortedBag`, and indexing it again per
// member pushed the index over its size cap.
const words = components.tokenizer.createTokenizer({ language: 'multilingual' });
const tokenizer = {
  language: words.language,
  normalizationCache: words.normalizationCache,
  tokenize(raw: string, language?: string, prop?: string, withCache?: boolean) {
    const tokens = words.tokenize(raw, language, prop, withCache);
    return [...tokens, ...tokens.flatMap((t) => (t.includes('.') ? t.split('.').filter(Boolean).slice(-1) : []))];
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

// A table cell's content repeats its table's padded header; the row alone,
// spaces squeezed, says the same in a fraction of the index.
function row(c: StructuredData['contents'][number]): StructuredData['contents'][number] {
  if (!(c as { table?: string }).table) return c;
  return { ...c, content: c.content.split('\n').at(-1)!.replace(/\s+/g, ' ') };
}

function rows(data: StructuredData): StructuredData {
  return { headings: data.headings, contents: data.contents.map(row) };
}

// Generated reference pages index their headings (one per item), the page
// intro, its "Start here" list and each item's "Elsewhere" row (the names
// other languages use) only: their full text made the index larger than
// the compiler. Core chapters also keep their table rows, where each method
// gets one line.
function brief(data: StructuredData, tables: boolean): StructuredData {
  return {
    headings: data.headings,
    contents: data.contents.flatMap((c) => {
      const r = row(c);
      if (!c.heading || c.heading === 'start-here') return [r];
      if (r !== c && (tables || /^\W*Elsewhere\b/.test(r.content))) return [r];
      return [];
    }),
  };
}

// Search matches `content` only, groups by `page_id` and filters by `tags`
// and `locale`; indexing every url and type for full text as well made a
// third of the index. createFromSource hands `schema` to the database
// (fumadocs-core 16) though its options type doesn't list it.
const fields = { schema: { content: 'string', page_id: 'string', tags: 'enum[]', locale: 'enum' } };

export const revalidate = false;
export const { staticGET: GET } = createFromSource(source, {
  // Results rank by relevance; no query sorts by a field, so skip the sort index.
  sort: { enabled: false },
  ...fields,
  tokenizer,
  async buildIndex(page) {
    const data = await structured(page);
    return {
      id: page.url,
      url: page.url,
      title: page.data.title ?? page.url,
      description: page.data.description,
      structuredData: page.url.startsWith('/docs/ref/') ? brief(data, page.url.startsWith('/docs/ref/core/')) : rows(data),
    };
  },
});
