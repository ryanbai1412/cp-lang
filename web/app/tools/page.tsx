import type { Metadata } from 'next';
import Link from 'next/link';
import { groups, tools } from '@/lib/tools';

export const metadata: Metadata = {
  title: 'Tools',
  description: 'Every cpl command: write, submit, test and debug.',
};

export default function Page() {
  const byName = new Map(tools.commands.map((c) => [c.name, c]));
  const listed = new Set(groups.flatMap((g) => g.names));
  const rest = tools.commands.filter((c) => !listed.has(c.name)).map((c) => c.name);
  const all = rest.length ? [...groups, { title: 'Other', names: rest }] : groups;
  return (
    <main className="container mx-auto max-w-4xl px-4 py-12">
      <p className="text-sm text-muted-foreground">Generated from {tools.version}</p>
      <h1 className="mt-2 text-3xl font-semibold tracking-tight">Tools</h1>
      <p className="mt-2 text-muted-foreground">
        One binary, <code>cpl</code>, does everything from checking a file to finding the smallest input where your
        solution and a brute disagree. Each page below is generated from <code>cpl &lt;command&gt; --help</code>.
      </p>
      <p className="mt-4 text-sm">
        <Link className="underline" href="/tools/judges">
          Judge profiles
        </Link>{' '}
        · {tools.judges.length} judges, the compiler and limits each build targets.
      </p>
      {all.map((g) => (
        <section key={g.title} className="mt-10">
          <h2 className="text-lg font-semibold">{g.title}</h2>
          <div className="mt-3 divide-y rounded-xl border">
            {g.names
              .map((n) => byName.get(n))
              .filter((c) => c !== undefined)
              .map((c) => (
                <Link
                  key={c.name}
                  href={`/tools/cli/${c.name}`}
                  className="flex flex-col gap-1 px-4 py-3 transition-colors hover:bg-accent sm:flex-row sm:gap-4"
                >
                  <code className="shrink-0 text-sm sm:w-64">cpl {c.usage}</code>
                  <span className="text-sm text-muted-foreground">
                    {c.summary.split('`').map((part, i) => (i % 2 ? <code key={i}>{part}</code> : part))}
                  </span>
                </Link>
              ))}
          </div>
        </section>
      ))}
    </main>
  );
}
