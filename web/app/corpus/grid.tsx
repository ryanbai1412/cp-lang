'use client';

import Link from 'next/link';
import { useMemo, useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface Item {
  problem: string;
  source: string;
  tags: string[];
  difficulty: string;
  cppLines: number;
  cpLines: number;
}

export function CorpusGrid({ entries }: { entries: Item[] }) {
  const [tag, setTag] = useState<string | null>(null);
  const tags = useMemo(() => {
    const c = new Map<string, number>();
    for (const e of entries) for (const t of e.tags) c.set(t, (c.get(t) ?? 0) + 1);
    return [...c].sort((a, b) => b[1] - a[1]).slice(0, 16);
  }, [entries]);
  const shown = tag ? entries.filter((e) => e.tags.includes(tag)) : entries;
  return (
    <>
      <div className="mt-6 flex flex-wrap gap-2">
        <Button size="sm" variant={tag === null ? 'default' : 'outline'} onClick={() => setTag(null)}>
          All {entries.length}
        </Button>
        {tags.map(([t, n]) => (
          <Button key={t} size="sm" variant={tag === t ? 'default' : 'outline'} onClick={() => setTag(t)}>
            {t} {n}
          </Button>
        ))}
      </div>
      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {shown.map((e) => (
          <Link key={e.problem} href={`/corpus/${e.problem}`} className="group">
            <Card className="h-full transition-colors group-hover:bg-accent">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="font-mono text-base">{e.problem}</CardTitle>
                  <Badge variant="secondary">{e.difficulty}</Badge>
                </div>
                <CardDescription className="line-clamp-2">{[e.source, ...e.tags].filter(Boolean).join(' · ')}</CardDescription>
                <p className="pt-1 font-mono text-xs text-muted-foreground">
                  C++ {e.cppLines} lines → CP Lang {e.cpLines}
                </p>
              </CardHeader>
            </Card>
          </Link>
        ))}
      </div>
    </>
  );
}
