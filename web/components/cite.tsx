import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import Link from 'next/link';

// Citations in the Language chapters. Each reads the cited file at build time
// and fails the build if the decision or the spec heading is gone, so a
// chapter can't keep pointing at something that changed.
const repo = join(process.cwd(), '..');
const blob = 'https://github.com/ryanbai1412/cp-lang/blob/main';

// GitHub's and Fumadocs' heading ids: lowercase, punctuation dropped, spaces to dashes.
const slug = (heading: string) =>
  heading
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .replace(/\s/g, '-');

const headings = (file: string) =>
  readFileSync(join(repo, file), 'utf8')
    .split('\n')
    .flatMap((l) => (/^#{1,6} /.test(l) ? [l.replace(/^#+ /, '').replace(/`/g, '').trim()] : []));

export function Decision({ n, children }: { n: number; children?: React.ReactNode }) {
  const heading = headings('decisions.md').find((h) => h.startsWith(`${n}. `));
  if (!heading) throw new Error(`<Decision n={${n}}>: decisions.md has no "## ${n}." entry`);
  const title = heading.slice(`${n}. `.length).replace(/\s*\([^)]*\d{4}-\d\d-\d\d[^)]*\)$/, '');
  return (
    <a href={`${blob}/decisions.md#${slug(heading)}`} title={title}>
      {children ?? `decision ${n}`}
    </a>
  );
}

/** `file` is under spec/, e.g. `core/05-ownership`; `section` is the heading text. */
export function Spec({ file, section, children }: { file: string; section?: string; children?: React.ReactNode }) {
  const path = `spec/${file}.md`;
  if (section && !headings(path).includes(section.replace(/`/g, ''))) {
    throw new Error(`<Spec file="${file}" section="${section}">: ${path} has no such heading`);
  }
  const href = file.startsWith('core/') ? `/docs/ref/${file}` : `${blob}/${path}`;
  const anchor = section ? `#${slug(section.replace(/`/g, ''))}` : '';
  return (
    <Link href={`${href}/${anchor}`}>
      {children ?? (section ? `${file} §${section.split(' ')[0].replace(/\.$/, '')}` : file)}
    </Link>
  );
}
