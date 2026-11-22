import type { StructuredData } from 'fumadocs-core/mdx-plugins';
import { createFromSource } from 'fumadocs-core/search/server';
import { source } from '@/lib/source';

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
function brief(data: StructuredData): StructuredData {
  return { headings: data.headings, contents: data.contents.filter((c) => !c.heading) };
}

export const revalidate = false;
export const { staticGET: GET } = createFromSource(source, {
  // Results rank by relevance; no query sorts by a field, so skip the sort index.
  sort: { enabled: false },
  async buildIndex(page) {
    const data = await structured(page);
    return {
      id: page.url,
      url: page.url,
      title: page.data.title ?? page.url,
      description: page.data.description,
      structuredData: page.url.startsWith('/docs/ref/') ? brief(data) : data,
    };
  },
});
