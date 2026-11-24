import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { tools } from '@/lib/tools';

type Props = { params: Promise<{ command: string }> };

export function generateStaticParams() {
  return tools.commands.map((c) => ({ command: c.name }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { command } = await params;
  const c = tools.commands.find((x) => x.name === command);
  if (!c) notFound();
  return { title: `cpl ${c.name}`, description: c.summary };
}

export default async function Page({ params }: Props) {
  const { command } = await params;
  const c = tools.commands.find((x) => x.name === command);
  if (!c) notFound();
  return (
    <main className="container mx-auto max-w-4xl px-4 py-12">
      <Link href="/tools" className="text-sm text-muted-foreground hover:underline">
        ← Tools
      </Link>
      <h1 className="mt-2 font-mono text-3xl font-semibold tracking-tight">cpl {c.name}</h1>
      <p className="mt-2 text-muted-foreground">{c.summary}</p>
      <pre tabIndex={0} className="mt-6 overflow-x-auto rounded-xl border bg-muted/30 p-4 text-sm">
        {c.help ?? `usage: cpl ${c.usage}`}
      </pre>
      <p className="mt-4 text-xs text-muted-foreground">Generated from {tools.version}.</p>
    </main>
  );
}
