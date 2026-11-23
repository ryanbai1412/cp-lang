import type { Metadata } from 'next';
import Link from 'next/link';
import { InlineMd } from '@/components/inline-md';
import { codes, families } from '@/lib/errors';

export const metadata: Metadata = {
  title: 'Errors',
  description: 'Every CP Lang error, warning and runtime trap: what it means, a program that reports it, and the fix.',
};

export default function Page() {
  const all = codes().filter((c) => !c.code.startsWith('L'));
  const lints = codes().length - all.length;
  return (
    <main className="container mx-auto max-w-5xl px-4 py-10 [&_code]:rounded [&_code]:bg-muted [&_code]:px-1 [&_code]:text-[0.9em]">
      <h1 className="text-3xl font-semibold tracking-tight">Errors</h1>
      <p className="mt-3 max-w-3xl text-muted-foreground">
        {all.length} codes for errors, warnings and runtime traps. A code never changes meaning; each page has a program
        that reports exactly it and the fix, both runnable. The same text is <code>cpl explain &lt;code&gt;</code>. The{' '}
        {lints} lint codes have their own pages under <Link className="underline" href="/lints">Lints</Link>.
      </p>
      <nav className="mt-6 flex flex-wrap gap-x-4 gap-y-1 text-sm">
        {families.map(([prefix, title]) => (
          <a key={prefix} className="underline" href={`#${prefix.toLowerCase()}`}>
            {title}
          </a>
        ))}
      </nav>
      {families.map(([prefix, title]) => {
        const list = all.filter((c) => c.code.startsWith(prefix));
        if (list.length === 0) return null;
        return (
          <section key={prefix} id={prefix.toLowerCase()} className="mt-10">
            <h2 className="text-xl font-semibold">
              {title} <span className="font-mono text-sm text-muted-foreground">{prefix}</span>
            </h2>
            <ul className="mt-3 divide-y rounded-xl border">
              {list.map((c) => (
                <li key={c.code} className="flex flex-col gap-1 px-4 py-3 sm:flex-row sm:gap-4">
                  <Link href={`/errors/${c.code}`} className="shrink-0 font-mono text-sm font-medium underline sm:w-16">
                    {c.code}
                  </Link>
                  <span className="min-w-0 text-sm">
                    <span className="font-medium">{c.title}.</span>{' '}
                    <span className="text-muted-foreground">
                      <InlineMd text={c.summary} />
                    </span>
                  </span>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </main>
  );
}
