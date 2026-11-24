import type { DocsLayoutProps } from "fumadocs-ui/layouts/notebook";
import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import type { HomeLayoutProps } from "fumadocs-ui/layouts/home";
import { DocsHeader, HomeHeader } from "@/components/site-header";

export const site = {
  name: "CP Lang",
  github: "https://github.com/ryanbai1412/cp-lang",
};

export const navLinks = [
  { text: "Learn", url: "/learn" },
  { text: "Docs", url: "/docs" },
  { text: "Playground", url: "/play" },
  { text: "Problems", url: "/corpus" },
  { text: "Tools", url: "/tools" },
  { text: "Performance", url: "/performance" },
  { text: "Install", url: "/setup" },
];

export function baseOptions(): BaseLayoutProps {
  return { githubUrl: site.github, links: navLinks };
}

// Both layouts render components/site-header.tsx, so every page has the same
// header. Docs and Learn keep the links in their sidebar for phones.
export const homeOptions = (): HomeLayoutProps => ({ ...baseOptions(), slots: { header: HomeHeader } });

export const notebookOptions = (): Omit<DocsLayoutProps, "tree"> => ({
  ...baseOptions(),
  nav: { mode: "top" },
  links: navLinks.map((l) => ({ ...l, on: "menu" as const })),
  slots: { header: DocsHeader },
});
