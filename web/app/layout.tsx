import './global.css';
import { GeistMono } from 'geist/font/mono';
import { GeistSans } from 'geist/font/sans';
import { RootProvider } from 'fumadocs-ui/provider/next';
import type { Metadata } from 'next';
import { TooltipProvider } from '@/components/ui/tooltip';
import type { ReactNode } from 'react';

export const metadata: Metadata = {
  title: { default: 'CP Lang: a language for competitive programming', template: '%s · CP Lang' },
  description:
    'CP Lang is a language for competitive programming that compiles to one readable C++17 file.',
};

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className={`${GeistSans.variable} ${GeistMono.variable}`} suppressHydrationWarning>
      <body className="flex min-h-screen flex-col font-sans antialiased">
        <RootProvider search={{ options: { type: 'static' } }}>
          <TooltipProvider>{children}</TooltipProvider>
        </RootProvider>
      </body>
    </html>
  );
}
