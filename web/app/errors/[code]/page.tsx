import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound, redirect } from 'next/navigation';
import { InlineMd } from '@/components/inline-md';
import { Runner } from '@/components/runner';
import { Badge } from '@/components/ui/badge';
import { codes, familyOf } from '@/lib/errors';

type Props = { params: Promise<{ code: string }> };

const catalogue = 'https://github.com/ryanbai1412/cp-lang/blob/main/spec/diagnostics.md';

export function generateStaticParams() {
  // Lint codes forward to their lint page, so any diagnostic can link here by code.
  return codes().map((c) => ({ code: c.code }));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { code } = await params;
  const c = codes().find((x) => x.code === code);
  return { title: c ? `${c.code}: ${c.title} · Errors` : 'Errors', description: c?.summary.replace(/`/g, '') };
}

export default async function Page({ params }: Props) {
  const { code } = await params;
  const c = codes().find((x) => x.code === code);
  if (!c) notFound();
  if (c.code.startsWith('L')) redirect(`/lints/${c.code}`);
  const family = familyOf(c.code);
  const runnable = c.trigger.trim() !== '' && ['check', 'run'].includes(c.mode);
  return (
    <main className="container mx-auto max-w-5xl px-4 py-10 [&_p_code]:rounded [&_p_code]:bg-muted [&_p_code]:px-1 [&_p_code]:text-[0.9em]">
      <Link href="/errors" className="text-sm text-muted-foreground hover:text-foreground">
        ← All errors
      </Link>
      <div className="mt-3 flex flex-wrap items-center gap-3">
        <h1 className="font-mono text-3xl font-semibold tracking-tight">{c.code}</h1>
        {family && (
          <Badge variant="outline">
            <Link href={`/errors#${family[0].toLowerCase()}`}>{family[1]}</Link>
          </Badge>
        )}
      </div>
      <p className="mt-2 text-lg font-medium">{c.title}</p>
      <p className="mt-1 text-muted-foreground">
        <InlineMd text={c.summary} />
      </p>
      <p className="mt-4 max-w-3xl">
        <InlineMd text={c.explanation} />
      </p>
      {runnable ? (
        <div className="mt-6 grid gap-4 lg:grid-cols-2">
          <div className="min-w-0 space-y-2">
            <h2 className="text-sm font-medium">Reports {c.code}</h2>
            <Runner source={c.trigger} stdin={c.input} status="rejects" compact />
          </div>
          <div className="min-w-0 space-y-2">
            <h2 className="text-sm font-medium">The fix</h2>
            <Runner source={c.fix} stdin={c.input} status="runs" compact />
          </div>
        </div>
      ) : (
        <p className="mt-6 text-sm text-muted-foreground">
          This code is {c.mode}: no program reports it today.
        </p>
      )}
      <p className="mt-8 text-sm text-muted-foreground">
        Normative list: the{' '}
        <a className="underline" href={catalogue}>
          diagnostics catalogue
        </a>
        . In a terminal: <code>cpl explain {c.code}</code>.
      </p>
    </main>
  );
}
