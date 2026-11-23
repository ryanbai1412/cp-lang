import { Markdown } from 'fumadocs-core/content/md';
import { remarkGfm } from 'fumadocs-core/mdx-plugins/remark-gfm';
import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { InlineMd } from '@/components/inline-md';
import { decisions, row } from '@/lib/decisions';

type El = {
  type: string;
  tagName?: string;
  value?: string;
  properties?: Record<string, unknown>;
  children?: El[];
};
const text = (n: El): string => n.value ?? (n.children ?? []).map(text).join('');

// Gives a table row or paragraph that starts a sub-decision (`| 1.7 |`, `26.4.`) the anchor it is cited by.
function rehypeRows(rows: Set<string>) {
  const walk = (n: El) => {
    if (n.tagName === 'tr' || n.tagName === 'p') {
      const cells = n.tagName === 'tr' ? (n.children ?? []).filter((c) => c.type === 'element') : [n];
      const lead = n.tagName === 'tr' ? `| ${text(cells[0] ?? { type: 'text' }).trim()} |` : text(n);
      const m = row.exec(lead);
      const num = m && (m[1] ?? m[2]);
      if (num && rows.delete(num))
        n.properties = {
          ...n.properties,
          id: `d-${num}`,
          className: ['scroll-mt-20'],
        };
    }
    n.children?.forEach(walk);
  };
  return () => (tree: El) => walk(tree);
}

export function generateStaticParams() {
  return decisions().map((d) => ({ slug: d.slug }));
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }): Promise<Metadata> {
  const { slug } = await params;
  const d = decisions().find((x) => x.slug === slug);
  return d ? { title: `Decision ${d.number}: ${d.title.replace(/`/g, '')}` } : {};
}

export default async function Page({ params }: { params: Promise<{ slug: string }> }) {
  const all = decisions();
  const { slug } = await params;
  const at = all.findIndex((x) => x.slug === slug);
  if (at < 0) notFound();
  const d = all[at];
  const rows = new Set(d.rows);
  const nav = (x: (typeof all)[number] | undefined, label: string) =>
    x && (
      <Link prefetch={false} className="underline" href={`/decisions/${x.slug}`}>
        {label} {x.number}
      </Link>
    );
  return (
    <main className="container mx-auto max-w-3xl px-4 py-10">
      <p className="text-sm text-muted-foreground">
        <Link prefetch={false} className="underline" href="/decisions">
          Design decisions
        </Link>
      </p>
      <h1 className="mt-2 text-2xl font-semibold tracking-tight [&_code]:text-[0.9em]">
        {d.number}. <InlineMd text={d.title} />
      </h1>
      <div className="prose mt-6 max-w-none [&_pre]:overflow-x-auto [&_table]:block [&_table]:overflow-x-auto">
        {d.parts.map((p, i) => (
          <div key={p.id ?? i} id={p.id} className="scroll-mt-20">
            <Markdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRows(rows)]}>
              {p.text}
            </Markdown>
          </div>
        ))}
      </div>
      <nav className="mt-10 flex justify-between text-sm">
        <span>{nav(all[at - 1], 'Previous:')}</span>
        <span>{nav(all[at + 1], 'Next:')}</span>
      </nav>
    </main>
  );
}
