import type { Metadata } from 'next';
import Link from 'next/link';
import { InlineMd } from '@/components/inline-md';
import { Badge } from '@/components/ui/badge';
import { type Group, groups, lints, principles } from '@/lib/lints';

export const metadata: Metadata = {
  title: 'Lints',
  description: 'Every CP Lang lint: what it flags, the rewrite, and why.',
};

export default function Page() {
  const all = lints();
  const families = [...new Set(all.flatMap((l) => (l.family ? [l.family] : [])))];
  return (
    <main className="container mx-auto max-w-5xl px-4 py-10 [&_code]:rounded [&_code]:bg-muted [&_code]:px-1 [&_code]:text-[0.9em]">
      <h1 className="text-3xl font-semibold tracking-tight">Lints</h1>
      <p className="mt-3 max-w-3xl text-muted-foreground">
        {all.length} lints, each with the code it flags, the rewrite and the reason. <code>cpl lint</code> and the
        editor show them; none ever fails <code>cpl build</code> or <code>cpl run</code>. Set a lint's level in{' '}
        <code>cp.toml</code> under <code>[lints]</code>, or silence one in code with <code>with lint(name: allow)</code>
        .
      </p>
      <nav className="mt-6 flex flex-wrap gap-x-4 gap-y-1 text-sm">
        <a className="underline" href="#by-group">
          By group
        </a>
        <a className="underline" href="#by-family">
          By family
        </a>
        <a className="underline" href="#by-practice">
          By practice
        </a>
      </nav>
      <h2 id="by-group" className="sr-only">
        By group
      </h2>
      {(Object.keys(groups) as Group[]).map((g) => {
        const list = all.filter((l) => l.group === g);
        if (list.length === 0) return null;
        return (
          <section key={g} className="mt-10">
            <h2 id={g} className="text-xl font-semibold">
              {groups[g].title} <span className="text-sm font-normal text-muted-foreground">{list.length}</span>
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              <InlineMd text={groups[g].practice} />
            </p>
            <ul className="mt-4 divide-y rounded-xl border [overflow-wrap:anywhere]">
              {list.map((l) => (
                <li key={l.name}>
                  <Link
                    href={`/lints/${l.name}`}
                    className="flex flex-col gap-1 px-4 py-3 hover:bg-muted/50 sm:flex-row sm:gap-4"
                  >
                    <span className="shrink-0 font-mono text-sm break-all sm:w-64">
                      {l.name} <span className="text-muted-foreground">{l.code}</span>
                    </span>
                    <span className="min-w-0 text-sm">
                      <span className="font-medium">
                        <InlineMd text={l.title} />
                      </span>
                      {l.level === 'allow' && (
                        <Badge variant="outline" className="ml-2">
                          off by default
                        </Badge>
                      )}
                      <span className="block text-muted-foreground">
                        <InlineMd text={l.summary} />
                      </span>
                    </span>
                  </Link>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
      <h2 id="by-family" className="mt-14 text-2xl font-semibold">
        By family
      </h2>
      <p className="mt-1 text-sm text-muted-foreground">
        The families of the idioms catalog, <code>spec/idioms/lints.md</code>.
      </p>
      {families.map((f) => (
        <section key={f} className="mt-6">
          <h3 className="font-semibold">{f}</h3>
          <Names list={all.filter((l) => l.family === f)} />
        </section>
      ))}
      <h2 id="by-practice" className="mt-14 text-2xl font-semibold">
        By practice
      </h2>
      <p className="mt-1 text-sm text-muted-foreground">
        The practices of <code>spec/idioms/principles.md</code> each lint serves.
      </p>
      {principles().map((p) => {
        const list = all.filter((l) => l.principles.some((q) => q.n === p.n));
        if (list.length === 0) return null;
        return (
          <section key={p.n} id={`practice-${p.n}`} className="mt-6">
            <h3 className="font-semibold">
              {p.n}. {p.title}
            </h3>
            <Names list={list} />
          </section>
        );
      })}
    </main>
  );
}

function Names({ list }: { list: ReturnType<typeof lints> }) {
  return (
    <ul className="mt-2 flex flex-wrap gap-x-4 gap-y-1 font-mono text-sm [overflow-wrap:anywhere]">
      {list.map((l) => (
        <li key={l.name}>
          <Link className="underline" href={`/lints/${l.name}`}>
            {l.name}
          </Link>
        </li>
      ))}
    </ul>
  );
}
