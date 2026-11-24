'use client';

import { useHomeLayout } from 'fumadocs-ui/layouts/home';
import { useNotebookLayout } from 'fumadocs-ui/layouts/notebook';
import { Menu } from 'lucide-react';
import Link from 'next/link';
import type { ComponentProps, ReactNode } from 'react';
import { NavLinks } from '@/components/nav-links';
import { PhoneInstall } from '@/components/phone-install';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { navLinks } from '@/lib/layout.shared';
import { cn } from '@/lib/utils';

type Slots = ReturnType<typeof useHomeLayout>['slots'];
type Items = ReturnType<typeof useHomeLayout>['navItems'];

const iconButton =
  'inline-flex size-9 items-center justify-center rounded-md text-fd-muted-foreground transition-colors hover:bg-fd-accent hover:text-fd-accent-foreground [&_svg]:size-5';

// The one header every page has, on both Fumadocs layouts: title and links,
// then search, theme and GitHub; on phones Install, search and one menu
// button. Only `menu` differs: Home pages list the links, Docs and Learn open
// their sidebar, which lists them above the page tree.
function Bar({ slots, items, menu }: { slots: Slots | ReturnType<typeof useNotebookLayout>['slots']; items: Items; menu: ReactNode }) {
  return (
    <div className="mx-auto flex h-14 w-full max-w-(--fd-layout-width) items-center gap-2 px-4 md:px-6">
      <Link href="/" className="inline-flex items-center gap-2.5 font-semibold">
        CP Lang
      </Link>
      <NavLinks />
      <PhoneInstall />
      <div className="ms-auto flex items-center gap-1.5 max-lg:hidden">
        {slots.searchTrigger && <slots.searchTrigger.full hideIfDisabled className="w-60 rounded-full ps-2.5" />}
        {slots.themeSwitch && <slots.themeSwitch />}
        {items
          .filter((i) => i.type === 'icon')
          .map((i) => (
            <a key={i.url} href={i.url} aria-label={i.label} className={iconButton} target="_blank" rel="noreferrer noopener">
              {i.icon}
            </a>
          ))}
      </div>
      <div className="flex items-center -me-1.5 lg:hidden">
        {slots.searchTrigger && <slots.searchTrigger.sm hideIfDisabled className="p-2" />}
        {menu}
      </div>
    </div>
  );
}

export function HomeHeader(props: ComponentProps<'header'>) {
  const { slots, navItems } = useHomeLayout();
  return (
    <header id="nd-nav" {...props} className={cn('sticky top-0 z-40 h-14 border-b bg-fd-background/80 backdrop-blur-lg', props.className)}>
      <Bar
        slots={slots}
        items={navItems}
        menu={
          <DropdownMenu>
            <DropdownMenuTrigger aria-label="Open the menu" className={iconButton}>
              <Menu />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {navLinks.map((l) => (
                <DropdownMenuItem key={l.url} asChild>
                  <Link href={l.url}>{l.text}</Link>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        }
      />
    </header>
  );
}

export function DocsHeader(props: ComponentProps<'header'>) {
  const { slots, navItems } = useNotebookLayout();
  return (
    <header
      id="nd-subnav"
      {...props}
      className={cn(
        'sticky top-(--fd-docs-row-1) z-10 border-b bg-fd-background/80 backdrop-blur-sm [grid-area:header] layout:[--fd-header-height:--spacing(14)]',
        props.className,
      )}
    >
      <Bar
        slots={slots}
        items={navItems}
        menu={
          <slots.sidebar.trigger aria-label="Open the menu" className={iconButton}>
            <Menu />
          </slots.sidebar.trigger>
        }
      />
    </header>
  );
}
