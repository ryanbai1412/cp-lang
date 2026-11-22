import type { DocsLayoutProps } from 'fumadocs-ui/layouts/notebook';
import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import { NavLinks } from '@/components/nav-links';

export const site = {
  name: 'CP Lang',
  github: 'https://github.com/ryanbai1412/cp-lang',
};

export const navLinks = [
  { text: 'Learn', url: '/learn' },
  { text: 'Docs', url: '/docs' },
  { text: 'Playground', url: '/play' },
  { text: 'Problems', url: '/corpus' },
  { text: 'Tools', url: '/tools' },
  { text: 'Performance', url: '/performance' },
  { text: 'Install', url: '/setup' },
];

export function baseOptions(): BaseLayoutProps {
  return {
    nav: { title: <span className="font-semibold">{site.name}</span> },
    githubUrl: site.github,
    links: navLinks,
  };
}

// The notebook layout puts its links right of a centred search. Docs and Learn
// show them after the title instead, as the landing does, and keep them in
// the phone menu; `global.css` moves the search next to the icons.
export function notebookOptions(): Omit<DocsLayoutProps, 'tree'> {
  const base = baseOptions();
  return {
    ...base,
    nav: { ...base.nav, mode: 'top', children: <NavLinks /> },
    links: navLinks.map((l) => ({ ...l, on: 'menu' as const })),
  };
}
