import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export const site = {
  name: 'CP Lang',
  github: 'https://github.com/ryanbai1412/cp-lang',
};

export function baseOptions(): BaseLayoutProps {
  return {
    nav: { title: <span className="font-semibold">{site.name}</span> },
    githubUrl: site.github,
    links: [
      { text: 'Learn', url: '/learn' },
      { text: 'Docs', url: '/docs' },
      { text: 'Playground', url: '/play' },
    ],
  };
}
