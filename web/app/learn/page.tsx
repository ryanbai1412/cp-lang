import type { Metadata } from 'next';
import Link from 'next/link';
import { Badge } from '@/components/ui/badge';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { tracks } from '@/lib/course';

export const metadata: Metadata = {
  title: 'Learn',
  description: 'Learn CP Lang by solving problems in the browser.',
};

export default function Page() {
  const all = tracks();
  return (
    <main className="container mx-auto max-w-5xl px-4 py-12">
      <h1 className="text-3xl font-semibold tracking-tight">Learn CP Lang by solving problems</h1>
      <p className="mt-2 text-muted-foreground">
        {all.reduce((n, t) => n + t.exercises.length, 0)} exercises in {all.length} tracks. Everything runs in your
        browser on the real compiler.
      </p>
      <div className="mt-8 grid gap-4 sm:grid-cols-2">
        {all.map((t, i) => (
          <Link key={t.slug} href={`/learn/${t.slug}/${t.exercises[0].id}`} className="group">
            <Card className="h-full transition-colors group-hover:bg-accent">
              <CardHeader>
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <span className="font-mono">{String(i + 1).padStart(2, '0')}</span>
                  <Badge variant="secondary">{t.level}</Badge>
                  <span>{t.exercises.length} exercises · {t.duration}</span>
                </div>
                <CardTitle>{t.title}</CardTitle>
                <CardDescription>{t.description}</CardDescription>
              </CardHeader>
            </Card>
          </Link>
        ))}
      </div>
    </main>
  );
}
