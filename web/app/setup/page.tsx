import type { Metadata } from 'next';
import { Checklist } from './checklist';

export const metadata: Metadata = {
  title: 'Set up CP Lang',
  description: 'Install cpl and the editor extension, pick your judge, and set up the contest loop.',
};

export default function Page() {
  return (
    <main className="container mx-auto max-w-3xl px-4 py-12">
      <p className="text-sm text-muted-foreground">Setup · about 10 minutes</p>
      <h1 className="mt-2 text-3xl font-semibold tracking-tight">Set up CP Lang on your machine</h1>
      <p className="mt-2 text-muted-foreground">
        The browser runs the same compiler, but contests go faster in your own editor. Tick each step as you go; this page
        remembers your progress.
      </p>
      <Checklist />
    </main>
  );
}
