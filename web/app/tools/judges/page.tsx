import type { Metadata } from 'next';
import Link from 'next/link';
import { Badge } from '@/components/ui/badge';
import { tools } from '@/lib/tools';

export const metadata: Metadata = {
  title: 'Judge profiles',
  description: 'The compiler, flags, stack and source limit each cpl build targets.',
};

export default function Page() {
  return (
    <main className="container mx-auto max-w-4xl px-4 py-12">
      <Link href="/tools" className="text-sm text-muted-foreground hover:underline">
        ← Tools
      </Link>
      <h1 className="mt-2 text-3xl font-semibold tracking-tight">Judge profiles</h1>
      <p className="mt-2 text-muted-foreground">
        A profile fixes the compiler, flags, stack size and source limit of a judge, so a local run behaves like the
        judge. Pick one with <code>cpl build --judge &lt;name&gt;</code>. Each profile says where its facts come from
        and which are unconfirmed.
      </p>
      <div className="mt-8 space-y-4">
        {tools.judges.map((j) => (
          <section key={j.name} id={j.name} className="rounded-xl border p-4">
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="font-mono text-lg font-semibold">{j.name}</h2>
              {j.default && <Badge variant="secondary">default</Badge>}
              <span className="text-sm text-muted-foreground">{j.compiler}</span>
            </div>
            {j.details[0] && (
              <pre tabIndex={0} className="mt-3 overflow-x-auto rounded-md bg-muted/40 p-2 text-xs">{j.details[0]}</pre>
            )}
            <ul className="mt-2 space-y-1 text-sm text-muted-foreground">
              {j.details.slice(1).map((d) => (
                <li key={d}>{d}</li>
              ))}
            </ul>
          </section>
        ))}
      </div>
      <p className="mt-4 text-xs text-muted-foreground">Generated from {tools.version}.</p>
    </main>
  );
}
