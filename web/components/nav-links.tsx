'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { navLinks } from '@/lib/layout.shared';

/** The landing's link row, for layouts whose header puts links elsewhere. */
export function NavLinks() {
  const path = usePathname();
  return (
    <div className="ms-6 flex items-center gap-6 text-sm max-lg:hidden">
      {navLinks.map((l) => (
        <Link
          key={l.url}
          href={l.url}
          data-active={path === l.url || path.startsWith(`${l.url}/`)}
          className="text-fd-muted-foreground transition-colors hover:text-fd-accent-foreground data-[active=true]:text-fd-primary"
        >
          {l.text}
        </Link>
      ))}
    </div>
  );
}
