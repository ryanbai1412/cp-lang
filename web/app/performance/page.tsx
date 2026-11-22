import type { Metadata } from 'next';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { bench, type Dist, histogram } from '@/lib/bench';
import { site } from '@/lib/layout.shared';

export const metadata: Metadata = {
  title: 'Performance',
  description: 'Runtime, memory and file size of CP Lang output against matched C++, generated from the benchmark.',
};

const x = (v: number) => `${v.toFixed(2)}x`;
const kb = (b: number) => `${(b / 1024).toFixed(1)} KB`;

export default function Page() {
  const { meta, summary, ports } = bench();
  const timed = ports.filter((p) => p.above_floor && p.time_ratio != null);
  const ratios = timed.map((p) => p.time_ratio!);
  const within = ratios.filter((r) => r <= 1.1).length;
  const buckets = histogram(ratios, [0, 0.5, 0.8, 0.95, 1.05, 1.1, 1.5, 2, 3, 5, Number.POSITIVE_INFINITY]);
  const max = Math.max(...buckets.map((b) => b.count));
  const slowest = [...timed].sort((a, b) => b.time_ratio! - a.time_ratio!).slice(0, 20);
  const t = summary.time_above_floor;
  return (
    <main className="container mx-auto max-w-5xl px-4 py-12">
      <h1 className="text-3xl font-semibold tracking-tight">Performance</h1>
      <p className="mt-2 max-w-3xl text-muted-foreground">
        Each port is a CP Lang rewrite of an accepted C++ submission from the corpus. Both programs run on the same
        maximum-size input where their outputs agree, compiled with <code>g++ -std=c++17 -O2</code>. This page is generated
        from the benchmark results; nothing here is typed by hand.
      </p>

      <Alert className="mt-6" variant={summary.target_met ? 'default' : 'destructive'}>
        <AlertTitle>{summary.target_met ? 'Target met' : 'Target not met'}</AlertTitle>
        <AlertDescription>
          The goal is a p95 runtime ratio of at most 1.1x. Today p95 is {x(t.p95)}; {within} of {t.n} timed ports (
          {Math.round((100 * within) / t.n)}%) are within 1.1x.
        </AlertDescription>
      </Alert>

      <div className="mt-8 grid gap-4 sm:grid-cols-3">
        <Stat title="Median runtime" value={x(t.median)} sub={`p90 ${x(t.p90)} · p95 ${x(t.p95)} · worst ${x(t.worst)}`} />
        <Stat title="Median peak memory" value={x(summary.rss_above_floor.median)} sub={`p95 ${x(summary.rss_above_floor.p95)}`} />
        <Stat title="Largest emitted file" value={kb(summary.max_emitted_bytes)} sub={`of ${summary.emitted_files} files; judges allow 64 KB`} />
      </div>

      <section className="mt-12">
        <h2 className="text-xl font-semibold tracking-tight">Runtime ratio, CP Lang / C++</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          {t.n} ports where either side runs at least {summary.floor_ms} ms. Below that a ratio mostly measures process start-up.
        </p>
        <div className="mt-4 space-y-1.5" role="img" aria-label="Histogram of runtime ratios">
          {buckets.map((b) => (
            <div key={b.lo} className="grid grid-cols-[110px_1fr_40px] items-center gap-3 text-sm">
              <span className="font-mono text-xs text-muted-foreground">
                {b.hi === Number.POSITIVE_INFINITY ? `≥ ${b.lo}x` : `${b.lo}–${b.hi}x`}
              </span>
              <div className="h-5 rounded-sm bg-muted">
                <div
                  className={b.hi <= 1.1 ? 'h-full rounded-sm bg-primary' : 'h-full rounded-sm bg-primary/40'}
                  style={{ width: `${(100 * b.count) / max}%` }}
                />
              </div>
              <span className="text-right font-mono text-xs">{b.count}</span>
            </div>
          ))}
        </div>
      </section>

      <section className="mt-12">
        <h2 className="text-xl font-semibold tracking-tight">The 20 slowest ports</h2>
        <div className="mt-4 overflow-x-auto rounded-xl border">
          <table className="w-full text-sm">
            <thead className="bg-muted/50 text-left text-muted-foreground">
              <tr>
                <th className="px-3 py-2 font-medium">Problem</th>
                <th className="px-3 py-2 font-medium">Runtime</th>
                <th className="px-3 py-2 font-medium">C++ → CP Lang</th>
                <th className="px-3 py-2 font-medium">Memory</th>
                <th className="px-3 py-2 font-medium">Emitted</th>
              </tr>
            </thead>
            <tbody>
              {slowest.map((p) => (
                <tr key={p.port} className="border-t">
                  <td className="px-3 py-2 font-mono text-xs">
                    <a className="underline-offset-4 hover:underline" href={`${site.github}/blob/main/${p.port}`}>
                      {p.problem}
                    </a>
                  </td>
                  <td className="px-3 py-2 font-mono">{x(p.time_ratio!)}</td>
                  <td className="px-3 py-2 font-mono text-xs text-muted-foreground">
                    {Math.round(p.orig_s! * 1000)} → {Math.round(p.port_s! * 1000)} ms
                  </td>
                  <td className="px-3 py-2 font-mono">{x(p.rss_ratio!)}</td>
                  <td className="px-3 py-2 font-mono text-xs">{kb(p.emitted_bytes)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <p className="mt-10 text-xs text-muted-foreground">
        Run at <code>{meta.commit}</code> with cpl {meta.cpl_version}, {meta.gxx}, {meta.runs} timed runs per binary,{' '}
        {meta.cpu} ({meta.cores} cores). {summary.measured} of {meta.problems} problems measured. Source:{' '}
        <a className="underline" href={`${site.github}/blob/main/bench/corpus-p95-report.md`}>bench/corpus-p95-report.md</a>.
      </p>
    </main>
  );
}

function Stat({ title, value, sub }: { title: string; value: string; sub: string }) {
  return (
    <Card>
      <CardHeader>
        <CardDescription>{title}</CardDescription>
        <CardTitle className="text-3xl font-semibold tabular-nums">{value}</CardTitle>
      </CardHeader>
      <CardContent className="text-xs text-muted-foreground">{sub}</CardContent>
    </Card>
  );
}
