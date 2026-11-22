import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Metadata } from 'next';
import Link from 'next/link';
import { Runner } from '@/components/runner';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export const metadata: Metadata = { title: 'Getting started', description: 'Your first CP Lang program, in the browser.' };

const start = join(process.cwd(), 'content/landing/start');
const first = readFileSync(`${start}.cp`, 'utf8');
const stdin = readFileSync(`${start}.in`, 'utf8');

const next = [
  { href: '/learn/from/cpp', title: 'Quickstart for your language', text: 'C++, Python, Java or Rust: six problems side by side.' },
  { href: '/learn', title: 'Learn', text: 'Chapters to read, and practice problems judged in your browser.' },
  { href: '/docs/guides/tour', title: 'The tour', text: 'The whole language in one sitting.' },
];

export default function Page() {
  return (
    <main className="container mx-auto max-w-3xl px-4 py-12">
      <p className="text-sm text-muted-foreground">Getting started · 5 minutes</p>
      <h1 className="mt-2 text-3xl font-semibold tracking-tight">Your first program</h1>
      <p className="mt-3 text-muted-foreground">
        CP Lang is a language for competitive programming. Each program compiles to one C++17 file that you submit to any
        judge. This page runs the same compiler in your browser.
      </p>
      <h2 className="mt-10 text-xl font-semibold">1. Run it</h2>
      <p className="mt-1 text-muted-foreground">
        The program reads n, then n numbers, and prints their sum. Press Run, then open the C++17 tab to see the file you
        would submit, with your source quoted at the top.
      </p>
      <Runner className="mt-4" source={first} stdin={stdin} status="runs" />
      <h2 className="mt-10 text-xl font-semibold">2. Break it on purpose</h2>
      <ul className="mt-2 list-disc space-y-2 pl-5 text-muted-foreground">
        <li>
          In the Input tab, change the first <code>5</code> to <code>6</code>. The run stops and names the line that
          expected six numbers.
        </li>
        <li>
          Change <code>a.sum()</code> to <code>a.summ()</code>. The error appears as you type and suggests <code>sum</code>.
        </li>
        <li>
          Change <code>1..=2e5</code> to <code>1..=3</code> and run. Input outside the range traps instead of being
          silently accepted.
        </li>
      </ul>
      <h2 className="mt-10 text-xl font-semibold">3. Where next</h2>
      <div className="mt-4 grid gap-3 sm:grid-cols-3">
        {next.map((n) => (
          <Link key={n.href} href={n.href} className="group">
            <Card className="h-full transition-colors group-hover:bg-accent">
              <CardHeader>
                <CardTitle className="text-base">{n.title}</CardTitle>
                <CardDescription>{n.text}</CardDescription>
              </CardHeader>
            </Card>
          </Link>
        ))}
      </div>
      <p className="mt-10 text-sm text-muted-foreground">
        To run programs locally, see <Link className="underline" href="/docs/guides/getting-started">Getting started &amp; CLI</Link>.
      </p>
    </main>
  );
}
