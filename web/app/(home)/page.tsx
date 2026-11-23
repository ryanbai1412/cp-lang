import { ArrowRight, BookOpen, Code2, GraduationCap } from "lucide-react";
import { readdirSync } from "node:fs";
import { join } from "node:path";
import Link from "next/link";
import type { ReactNode } from "react";
import { Runner } from "@/components/runner";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { bench } from "@/lib/bench";
import { corpus } from "@/lib/corpus";
import {
  debugDemo,
  heroPrograms,
  libraryPrograms,
  libraryShowcases,
  type Live,
  type Showcase,
  syntaxShowcases,
} from "@/lib/landing";

const starts = [
  {
    href: "/learn/intro",
    icon: GraduationCap,
    title: "Learn it in 30 minutes",
    text: "One page, every example runs in the browser.",
  },
  {
    href: "/learn/from/cpp",
    icon: Code2,
    title: "Coming from C++, Python, Java or Rust",
    text: "Six problems, side by side with what you know.",
  },
  {
    href: "/docs",
    icon: BookOpen,
    title: "Look something up",
    text: "Guides, the language reference and the library.",
  },
];

const pillars = [
  {
    href: "#syntax",
    title: "Clean, expressive syntax",
    text: "The code reads like the editorial.",
  },
  {
    href: "#library",
    title: "A powerful standard library",
    text: "Hard problems in a dozen lines.",
  },
  {
    href: "#debugging",
    title: "Easy debugging",
    text: "Stress test, shrink, then step through time.",
  },
];

function LiveTabs({ items }: { items: Live[] }) {
  return (
    <Tabs defaultValue={items[0].id} className="min-w-0">
      <TabsList className="max-w-full flex-wrap justify-start group-data-[orientation=horizontal]/tabs:h-auto">
        {items.map((s) => (
          <TabsTrigger key={s.id} value={s.id}>
            {s.title}
          </TabsTrigger>
        ))}
      </TabsList>
      {items.map((s) => (
        <TabsContent key={s.id} value={s.id} className="space-y-3">
          {s.status === "runs" ? (
            <Runner
              source={s.source}
              stdin={s.stdin}
              status="runs"
              caption={s.caption}
            />
          ) : (
            <Static
              html={s.html}
              caption={s.caption}
              bar={<StatusBadge status={s.status} />}
            />
          )}
          {s.cpp && (
            <details className="rounded-xl border bg-muted/30">
              <summary className="cursor-pointer px-4 py-2 text-sm text-muted-foreground">
                Compare with the C++ ({s.cpp.split("\n").length} lines against{" "}
                {s.source.trimEnd().split("\n").length})
              </summary>
              <pre className="overflow-x-auto border-t p-4 font-mono text-xs leading-relaxed">
                {s.cpp}
              </pre>
            </details>
          )}
        </TabsContent>
      ))}
    </Tabs>
  );
}

function Static({
  html,
  caption,
  bar,
}: {
  html: string;
  caption: string;
  bar: ReactNode;
}) {
  return (
    <div className="overflow-hidden rounded-xl border bg-card shadow-sm">
      <div className="flex items-center gap-2 border-b px-3 py-2 text-sm text-muted-foreground">
        {bar}
      </div>
      <div
        className="overflow-x-auto p-4 text-[13px] leading-relaxed [&_pre]:!bg-transparent"
        dangerouslySetInnerHTML={{ __html: html }}
      />
      <p className="border-t px-3 py-2 text-sm text-muted-foreground">
        {caption}
      </p>
    </div>
  );
}

function Showcases({ items }: { items: Showcase[] }) {
  return (
    <Tabs defaultValue={items[0].id} className="min-w-0">
      <TabsList className="max-w-full flex-wrap justify-start group-data-[orientation=horizontal]/tabs:h-auto">
        {items.map((s) => (
          <TabsTrigger key={s.id} value={s.id}>
            {s.title}
          </TabsTrigger>
        ))}
      </TabsList>
      {items.map((s) => (
        <TabsContent key={s.id} value={s.id}>
          <Static
            html={s.html}
            caption={s.caption}
            bar={
              <>
                <span>{s.lines} lines</span>
                <span className="ml-auto flex items-center gap-3">
                  {s.href && (
                    <Link href={s.href} className="hover:underline">
                      Compare with the C++{" "}
                      <ArrowRight className="inline size-3.5" />
                    </Link>
                  )}
                  <StatusBadge status={s.status} />
                </span>
              </>
            }
          />
        </TabsContent>
      ))}
    </Tabs>
  );
}

export default async function Home() {
  const [syntax, library, hero, lib] = await Promise.all([
    syntaxShowcases(),
    libraryShowcases(),
    heroPrograms(),
    libraryPrograms(),
  ]);
  const debug = debugDemo();
  const entries = corpus();
  const inGallery = new Set(entries.map((e) => e.problem));
  for (const s of library) if (!inGallery.has(s.id)) s.href = undefined;
  const modules = readdirSync(
    join(process.cwd(), "content/docs/ref/std"),
  ).filter((f) => /\.mdx?$/.test(f) && !f.startsWith("index.")).length;
  const { ports } = bench();
  const ratios = ports
    .filter((p) => p.above_floor && p.time_ratio != null)
    .map((p) => p.time_ratio!)
    .sort((a, b) => a - b);
  const median = ratios[Math.floor(ratios.length / 2)];
  const p95 = ratios[Math.ceil(0.95 * ratios.length) - 1];
  const problems = entries.length;
  return (
    <main className="container mx-auto max-w-6xl px-4">
      {/* On phones the left column's parts become grid items, so the program
          comes right after the one-line pitch and shows on the first screen. */}
      <section
        id="syntax"
        className="grid items-start gap-6 py-8 lg:grid-cols-[1fr_1.3fr] lg:gap-10 lg:py-16"
      >
        <div className="contents lg:block lg:space-y-6 lg:pt-10">
          <div className="space-y-4 lg:space-y-6">
            <p className="text-sm text-muted-foreground">Beta</p>
            <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">
              Write the editorial, not the boilerplate.
            </h1>
            <p className="text-lg text-muted-foreground">
              CP Lang is a language for competitive programming. It compiles to
              one readable C++17 file you submit anywhere.
            </p>
          </div>
          <ul className="order-3 space-y-3">
            {pillars.map((p) => (
              <li key={p.href}>
                <a href={p.href} className="group block">
                  <span className="font-medium group-hover:underline">
                    {p.title}.
                  </span>{" "}
                  <span className="text-muted-foreground">{p.text}</span>
                </a>
              </li>
            ))}
          </ul>
          <div className="order-4 flex flex-wrap gap-3">
            <Button size="lg" asChild>
              <Link href="/learn/intro">
                Learn it in 30 minutes <ArrowRight />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/play">Open the playground</Link>
            </Button>
          </div>
        </div>
        <div className="order-2 min-w-0 lg:order-none">
          <LiveTabs items={hero} />
        </div>
      </section>

      <section id="library" className="border-t py-16">
        <div className="mb-8 max-w-2xl space-y-2">
          <h2 className="text-2xl font-semibold tracking-tight">
            A standard library built for contests
          </h2>
          <p className="text-muted-foreground">
            Segment trees, shortest paths, union-find and modular arithmetic are
            one line each.{" "}
            {/* The library index is a 2,600-row table; prefetching it from here
                cost `/` more than its own page. */}
            <Link
              href="/docs/ref/std"
              prefetch={false}
              className="text-foreground hover:underline"
            >
              Browse the library ({modules} modules){" "}
              <ArrowRight className="inline size-3.5" />
            </Link>
          </p>
        </div>
        <LiveTabs items={lib} />
      </section>

      <section id="debugging" className="border-t py-16">
        <div className="mb-8 max-w-2xl space-y-2">
          <h2 className="text-2xl font-semibold tracking-tight">
            Find the bug in seconds
          </h2>
          <p className="text-muted-foreground">
            This solution has a bug. The brute force and the generator live in
            the same file, so one command finds a failing input, shrinks it, and
            names the first value that differs. Then you step through both runs
            side by side.
          </p>
        </div>
        <div className="grid gap-6 lg:grid-cols-2">
          <Runner
            source={debug.source}
            stdin={debug.stdin}
            status="runs"
            caption="Run it on the shrunk case: it prints 1, the answer is 2."
          />
          <div className="min-w-0 space-y-6">
            <div className="overflow-hidden rounded-xl border bg-card shadow-sm">
              <p className="border-b px-3 py-2 font-mono text-xs">
                $ {debug.stress.command}
              </p>
              <pre className="overflow-x-auto p-3 font-mono text-xs leading-relaxed">
                {debug.stress.output}
              </pre>
              <p className="border-t px-3 py-2 text-sm text-muted-foreground">
                Real output from today&apos;s compiler, regenerated on every
                build.
              </p>
            </div>
          </div>
        </div>
        <div className="mt-6 overflow-hidden rounded-xl border bg-card shadow-sm">
          <div className="flex items-center gap-2 border-b px-3 py-2 text-sm">
            <span className="font-medium">
              Time-travel trace of the shrunk case
            </span>
            <span className="text-muted-foreground">
              solution and brute, step by step
            </span>
            <a
              href="/trace-demo.html"
              className="ml-auto text-muted-foreground hover:underline"
            >
              Open full screen <ArrowRight className="inline size-3.5" />
            </a>
          </div>
          <iframe
            src="/trace-demo.html"
            title="Trace of the failing case"
            className="h-[640px] w-full"
            loading="lazy"
          />
        </div>
      </section>

      <section id="solutions" className="border-t py-16">
        <div className="mb-8 max-w-2xl space-y-2">
          <h2 className="text-2xl font-semibold tracking-tight">
            Whole olympiad solutions
          </h2>
          <p className="text-muted-foreground">
            Complete solutions to real olympiad problems, next to more of the
            syntax they rely on. Each carries its status: the ones the compiler
            does not run yet have no Run button.
          </p>
        </div>
        <div className="grid gap-8 lg:grid-cols-2">
          <Showcases items={library} />
          <Showcases items={syntax} />
        </div>
      </section>

      <section className="grid gap-4 border-t py-16 md:grid-cols-3">
        {starts.map(({ href, icon: Icon, title, text }) => (
          <Link key={href} href={href} className="group">
            <Card className="h-full transition-colors group-hover:bg-accent">
              <CardHeader>
                <Icon className="size-5 text-muted-foreground" />
                <CardTitle>{title}</CardTitle>
                <CardDescription>{text}</CardDescription>
              </CardHeader>
            </Card>
          </Link>
        ))}
      </section>

      <section className="grid gap-4 border-t py-16 md:grid-cols-2">
        <Link href="/performance" className="group">
          <Card className="h-full transition-colors group-hover:bg-accent">
            <CardHeader>
              <CardTitle>How fast is it?</CardTitle>
              <CardDescription>
                Across {ratios.length} timed ports, the generated C++ runs at a
                median {median.toFixed(2)}x of matched hand-written C++, and{" "}
                {p95.toFixed(2)}x at p95. The goal is 1.1x; it isn&apos;t met
                yet. See the full distribution{" "}
                <ArrowRight className="inline size-3.5" />
              </CardDescription>
            </CardHeader>
          </Card>
        </Link>
        <Link href="/corpus" className="group">
          <Card className="h-full transition-colors group-hover:bg-accent">
            <CardHeader>
              <CardTitle>{problems} real problems</CardTitle>
              <CardDescription>
                Accepted C++ next to its CP Lang rewrite, with notes, linking to
                each judge&apos;s statement. Browse the gallery{" "}
                <ArrowRight className="inline size-3.5" />
              </CardDescription>
            </CardHeader>
          </Card>
        </Link>
      </section>

      <footer className="flex flex-wrap gap-x-6 gap-y-2 border-t py-8 text-sm text-muted-foreground">
        <span>CP Lang is in beta.</span>
        <Link className="hover:underline" href="/setup">
          Set up
        </Link>
        <Link className="hover:underline" href="/tools">
          Tools
        </Link>
        <Link className="hover:underline" href="/docs/releases">
          Release notes
        </Link>
        <Link className="hover:underline" href="/llms.txt">
          llms.txt
        </Link>
        <a
          className="hover:underline"
          href="https://github.com/ryanbai1412/cp-lang"
        >
          GitHub
        </a>
      </footer>
    </main>
  );
}
