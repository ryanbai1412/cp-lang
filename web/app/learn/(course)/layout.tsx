import { DocsLayout } from 'fumadocs-ui/layouts/notebook';
import type { ReactNode } from 'react';
import { baseOptions } from '@/lib/layout.shared';
import { learnSource } from '@/lib/source';

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <DocsLayout tree={learnSource.getPageTree()} {...baseOptions()} nav={{ ...baseOptions().nav, mode: 'top' }}>
      {children}
    </DocsLayout>
  );
}
