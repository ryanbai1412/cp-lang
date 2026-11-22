import { DocsLayout } from 'fumadocs-ui/layouts/notebook';
import type { ReactNode } from 'react';
import { notebookOptions } from '@/lib/layout.shared';
import { learnSource } from '@/lib/source';

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <DocsLayout tree={learnSource.getPageTree()} {...notebookOptions()}>
      {children}
    </DocsLayout>
  );
}
