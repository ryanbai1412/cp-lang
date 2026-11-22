import { ArrowRight, BookOpen, Bug, FlaskConical, Gauge, GraduationCap, Code2 } from 'lucide-react';
import Link from 'next/link';
import { Runner } from '@/components/runner';
import { Button } from '@/components/ui/button';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { landingSnippets } from '@/lib/landing';

const starts = [
  { href: '/learn/start', icon: GraduationCap, title: 'Solve your first problem', text: 'Two minutes, in the browser, no install.' },
  { href: '/learn/from/cpp', icon: Code2, title: 'Coming from C++, Python, Java or Rust', text: 'Six problems, side by side with what you know.' },
  { href: '/docs', icon: BookOpen, title: 'Look something up', text: 'Guides, the language reference and the library.' },
];

const tools = [
  { icon: FlaskConical, title: 'Stress testing', text: 'cpl stress runs your solution against a brute and shrinks the failing case.' },
  { icon: Bug, title: 'Traces', text: 'Step through a run and see every value in source terms.' },
  { icon: Gauge, title: 'explain', text: 'See the widths, layouts and costs the compiler chose.' },
];

export default function Home() {
  const snippets = landingSnippets();
  return (
    <main className="container mx-auto max-w-6xl px-4">
      <section className="grid items-center gap-10 py-16 lg:grid-cols-[1fr_1.2fr]">
        <div className="space-y-6">
          <p className="text-sm text-muted-foreground">Beta · CP Lang 0.3</p>
          <h1 className="text-4xl font-semibold tracking-tight sm:text-5xl">Write the idea. Submit fast C++17.</h1>
          <p className="text-lg text-muted-foreground">
            CP Lang is a language for competitive programming. Types carry what the statement tells you, common bugs
            can&apos;t be written, and every program compiles to one readable C++17 file you submit anywhere.
          </p>
          <div className="flex flex-wrap gap-3">
            <Button size="lg" asChild>
              <Link href="/learn/start">
                Start with a problem <ArrowRight />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/play">Open the playground</Link>
            </Button>
          </div>
        </div>
        <Tabs defaultValue={snippets[0].id}>
          <TabsList>
            {snippets.map((s) => (
              <TabsTrigger key={s.id} value={s.id}>
                {s.title}
              </TabsTrigger>
            ))}
          </TabsList>
          {snippets.map((s) => (
            <TabsContent key={s.id} value={s.id}>
              <Runner source={s.source} stdin={s.stdin} status={s.status} caption={s.caption} />
            </TabsContent>
          ))}
        </Tabs>
      </section>

      <section className="grid gap-4 pb-16 md:grid-cols-3">
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

      <section className="border-t py-16">
        <h2 className="mb-8 text-2xl font-semibold tracking-tight">Doubt to evidence, fast</h2>
        <div className="grid gap-6 md:grid-cols-3">
          {tools.map(({ icon: Icon, title, text }) => (
            <div key={title} className="space-y-2">
              <Icon className="size-5 text-muted-foreground" />
              <h3 className="font-medium">{title}</h3>
              <p className="text-sm text-muted-foreground">{text}</p>
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}
