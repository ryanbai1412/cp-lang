import { ChevronLeft, ChevronRight } from 'lucide-react';
import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { Exercise } from '@/components/exercise';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { findExercise, tracks } from '@/lib/course';

type Props = { params: Promise<{ track: string; exercise: string }> };

export function generateStaticParams() {
  return tracks().flatMap((t) => t.exercises.map((e) => ({ track: t.slug, exercise: e.id })));
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { track, exercise } = await params;
  const found = findExercise(track, exercise);
  if (!found) notFound();
  return { title: `${found.exercise.title} · ${found.track.title}`, description: found.exercise.prompt };
}

export default async function Page({ params }: Props) {
  const { track, exercise } = await params;
  const found = findExercise(track, exercise);
  if (!found) notFound();
  const { track: t, exercise: e, index, prev, next } = found;
  return (
    <main className="container mx-auto grid max-w-6xl gap-8 px-4 py-10 lg:grid-cols-[220px_1fr]">
      <nav aria-label={t.title} className="hidden lg:block">
        <Link href="/learn" className="text-sm text-muted-foreground hover:text-foreground">
          ← All tracks
        </Link>
        <p className="mt-4 mb-2 text-sm font-medium">{t.title}</p>
        <ol className="space-y-1 text-sm">
          {t.exercises.map((x, i) => (
            <li key={x.id}>
              <Link
                href={`/learn/${t.slug}/${x.id}`}
                aria-current={x.id === e.id ? 'page' : undefined}
                className="block rounded-md px-2 py-1 text-muted-foreground hover:bg-accent aria-[current=page]:bg-accent aria-[current=page]:text-foreground"
              >
                {i + 1}. {x.title}
              </Link>
            </li>
          ))}
        </ol>
      </nav>
      <article className="min-w-0">
        <div className="mb-2 flex items-center gap-2 text-sm text-muted-foreground">
          <span>{t.title}</span>
          <span>·</span>
          <span>
            {index + 1} of {t.exercises.length}
          </span>
          <Badge variant="outline">{{ code: 'Write code', output: 'Predict output', choice: 'Choose' }[e.kind]}</Badge>
        </div>
        <h1 className="mb-6 text-3xl font-semibold tracking-tight">{e.title}</h1>
        <Exercise key={e.id} exercise={e} />
        <div className="mt-10 flex justify-between border-t pt-6">
          {prev ? (
            <Button variant="ghost" asChild>
              <Link href={`/learn/${t.slug}/${prev.id}`}>
                <ChevronLeft /> {prev.title}
              </Link>
            </Button>
          ) : (
            <span />
          )}
          {next && (
            <Button variant="outline" asChild>
              <Link href={`/learn/${t.slug}/${next.id}`}>
                {next.title} <ChevronRight />
              </Link>
            </Button>
          )}
        </div>
      </article>
    </main>
  );
}
