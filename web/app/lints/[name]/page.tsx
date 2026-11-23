import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound, redirect } from 'next/navigation';
import { InlineMd } from '@/components/inline-md';
import { Runner } from '@/components/runner';
import { Badge } from '@/components/ui/badge';
import { groups, lints } from '@/lib/lints';

type Props = { params: Promise<{ name: string }> };

const repo = 'https://github.com/ryanbai1412/cp-lang/tree/main';
const levels = { deny: 'deny: `cpl lint` exits 1', warn: 'warn', allow: 'off by default' };

export function generateStaticParams() {
  // `/lints/L0501` forwards to `/lints/search_loop`, so a diagnostic can link by its code.
  return lints().flatMap((l) => [{ name: l.name }, { name: l.code }]);
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { name } = await params;
  const l = lints().find((x) => x.name === name);
  return { title: l ? `${l.name} (${l.code}) · Lints` : 'Lints', description: l?.summary.replace(/`/g, '') };
}

export default async function Page({ params }: Props) {
  const { name } = await params;
  const byCode = lints().find((x) => x.code === name);
  if (byCode) redirect(`/lints/${byCode.name}`);
  const l = lints().find((x) => x.name === name);
  if (!l) notFound();
  return (
    <main className="container mx-auto max-w-5xl px-4 py-10 [&_p_code]:rounded [&_p_code]:bg-muted [&_p_code]:px-1 [&_p_code]:text-[0.9em]">
      <Link href="/lints" className="text-sm text-muted-foreground hover:text-foreground">
        ← All lints
      </Link>
      <div className="mt-3 flex flex-wrap items-center gap-3">
        <h1 className="font-mono text-3xl font-semibold tracking-tight break-all">{l.name}</h1>
        <Badge variant="secondary">{l.code}</Badge>
        <Badge variant="outline">
          <Link href={`/lints#${l.group}`}>{groups[l.group].title}</Link>
        </Badge>
        <Badge variant="outline">
          <InlineMd text={levels[l.level]} />
        </Badge>
      </div>
      <p className="mt-2 text-lg font-medium">
        <InlineMd text={l.title} />
      </p>
      <p className="mt-1 text-muted-foreground">
        <InlineMd text={l.summary} />
      </p>
      <div className="mt-6 grid gap-4 lg:grid-cols-2">
        <Pane title="Flagged" source={l.before} run={l.runs.before} />
        <Pane title="Write instead" source={l.after} run={l.runs.after} />
      </div>
      <h2 className="mt-8 text-xl font-semibold">Why</h2>
      <p className="mt-2 max-w-3xl leading-7">
        <InlineMd text={l.explain} />
      </p>
      <h2 className="mt-8 text-xl font-semibold">The practice it serves</h2>
      {l.principles.length > 0 ? (
        <ul className="mt-2 list-disc pl-6 leading-7">
          {l.principles.map((p) => (
            <li key={p.n}>
              <Link className="underline" href={`/lints#practice-${p.n}`}>
                {p.n}. {p.title}
              </Link>
            </li>
          ))}
        </ul>
      ) : null}
      <p className="mt-2 max-w-3xl leading-7">
        <InlineMd text={l.practice ?? groups[l.group].practice} />
        {l.family && (
          <>
            {' '}
            Family:{' '}
            <Link className="underline" href="/lints#by-family">
              {l.subfamily ?? l.family}
            </Link>
            .
          </>
        )}
      </p>
      <h2 className="mt-8 text-xl font-semibold">Not flagged</h2>
      <p className="mt-2 text-sm text-muted-foreground">A near miss: close to the pattern, but not it.</p>
      <div className="mt-3">
        <Pane title="Near miss" source={l.nearMiss} run={l.runs.nearMiss} />
      </div>
      <h2 className="mt-8 text-xl font-semibold">Turn it off</h2>
      <p className="mt-2 max-w-3xl leading-7">
        For one block, wrap it in <code>{`with lint(${l.name}: allow):`}</code>. For a project, add{' '}
        <code>{`${l.name} = "allow"`}</code> under <code>[lints]</code> in <code>cp.toml</code>.
      </p>
      <p className="mt-8 text-sm text-muted-foreground">
        Generated from{' '}
        <a className="underline" href={`${repo}/${l.dir}`}>
          {l.dir}
        </a>
        , the same record the linter is built from.
      </p>
    </main>
  );
}

function Pane({ title, source, run }: { title: string; source: string; run: boolean }) {
  return (
    <section className="min-w-0 space-y-2">
      <h3 className="text-sm font-medium">{title}</h3>
      <Runner source={source} compact lint run={run} />
    </section>
  );
}
