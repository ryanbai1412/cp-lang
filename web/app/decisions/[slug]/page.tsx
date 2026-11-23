import { Markdown } from 'fumadocs-core/content/md';
import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { InlineMd } from '@/components/inline-md';
import { decisions } from '@/lib/decisions';

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
            <Markdown>{p.text}</Markdown>
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
