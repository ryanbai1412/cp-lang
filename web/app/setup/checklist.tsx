'use client';

import { Check } from 'lucide-react';
import Link from 'next/link';
import { type ReactNode, useEffect, useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/utils';

const key = 'cpl-setup';

function Code({ children }: { children: string }) {
  return <pre className="mt-2 overflow-x-auto rounded-md border bg-muted/40 p-3 text-xs">{children}</pre>;
}

function Soon() {
  return (
    <Badge variant="outline" className="ml-2 align-middle text-[10px]">
      not published yet
    </Badge>
  );
}

const editors: { id: string; name: string; body: ReactNode }[] = [
  {
    id: 'vscode',
    name: 'VS Code',
    body: (
      <>
        <p>
          Marketplace and Open VSX (Cursor, Windsurf, VSCodium) installs <Soon />. Until then, package it from the
          repository:
        </p>
        <Code>{'cd editors/vscode\nnpm install\nnpm run package\ncode --install-extension cp-lang.vsix'}</Code>
      </>
    ),
  },
  {
    id: 'neovim',
    name: 'Neovim',
    body: (
      <>
        <p>Neovim 0.10 or later. Add the folder to the runtime path; it starts `cpl lsp` for every `.cp` buffer.</p>
        <Code>{"vim.opt.rtp:append('/path/to/cp-lang/editors/neovim')\nvim.g.cpl_path = 'cpl'"}</Code>
      </>
    ),
  },
  {
    id: 'helix',
    name: 'Helix',
    body: (
      <>
        <p>
          Append <code>editors/helix/languages.toml</code> to <code>~/.config/helix/languages.toml</code>, then:
        </p>
        <Code>
          {
            'mkdir -p ~/.config/helix/runtime/queries\nln -s /path/to/cp-lang/editors/helix/runtime/queries/cp ~/.config/helix/runtime/queries/cp\nhx --grammar build\nhx --health cp'
          }
        </Code>
      </>
    ),
  },
  {
    id: 'zed',
    name: 'Zed',
    body: (
      <p>
        Registry install <Soon />. Until then, run <code>zed: install dev extension</code> and pick{' '}
        <code>editors/zed</code> (needs Rust with the <code>wasm32-wasip1</code> target).
      </p>
    ),
  },
  {
    id: 'emacs',
    name: 'Emacs',
    body: (
      <Code>
        {
          "(add-to-list 'load-path \"/path/to/cp-lang/editors/emacs\")\n(require 'cp-mode)\n(add-hook 'cp-mode-hook #'eglot-ensure)"
        }
      </Code>
    ),
  },
  {
    id: 'jetbrains',
    name: 'JetBrains',
    body: (
      <p>
        Marketplace install <Soon />. Until then, build <code>editors/jetbrains</code> with <code>./gradlew buildPlugin</code>{' '}
        and install the zip with Settings → Plugins → Install Plugin from Disk (2024.3 or later).
      </p>
    ),
  },
];

const steps: { id: string; title: string; body: ReactNode }[] = [
  {
    id: 'install',
    title: 'Install cpl',
    body: (
      <>
        <p>
          Prebuilt binaries and an install script <Soon />. Today, build from source with Rust:
        </p>
        <Code>{'git clone https://github.com/ryanbai1412/cp-lang\ncd cp-lang\ncargo install --path crates/cpl-cli\ncpl --version'}</Code>
      </>
    ),
  },
  {
    id: 'editor',
    title: 'Install the editor extension',
    body: (
      <>
        <p>
          Every editor talks to the same language server, <code>cpl lsp</code>: diagnostics as you type, hover, completion,
          go to definition, rename and quick fixes.
        </p>
        <Tabs defaultValue="vscode" className="mt-3">
          <TabsList className="flex-wrap">
            {editors.map((e) => (
              <TabsTrigger key={e.id} value={e.id}>
                {e.name}
              </TabsTrigger>
            ))}
          </TabsList>
          {editors.map((e) => (
            <TabsContent key={e.id} value={e.id} className="text-sm text-muted-foreground">
              {e.body}
            </TabsContent>
          ))}
        </Tabs>
      </>
    ),
  },
  {
    id: 'verify',
    title: 'Check it works',
    body: (
      <>
        <p>
          Open a <code>.cp</code> file and type <code>let x = </code>; an error should appear as you type. Then:
        </p>
        <Code>{'echo \'print("hi")\' > hi.cp\ncpl run hi.cp'}</Code>
      </>
    ),
  },
  {
    id: 'judge',
    title: 'Pick your judge',
    body: (
      <>
        <p>
          A judge profile fixes the compiler, flags, stack and source limit, so local runs behave like the judge. See{' '}
          <Link className="underline" href="/tools/judges">
            all profiles
          </Link>
          .
        </p>
        <Code>{'cpl judges\ncpl build --judge codeforces a.cp'}</Code>
      </>
    ),
  },
  {
    id: 'loop',
    title: 'Set up the contest loop',
    body: (
      <>
        <p>Create a problem, keep its tests running on every save, stress it against a brute, then check it&apos;s ready.</p>
        <Code>{'cpl new a\ncpl watch a\ncpl stress a.cp\ncpl submit a.cp'}</Code>
        <p className="mt-2">
          One-click problem import from Competitive Companion <Soon />.
        </p>
      </>
    ),
  },
  {
    id: 'ai',
    title: 'Optional: point your AI assistant at the docs',
    body: (
      <p>
        <Link className="underline" href="/llms.txt">
          /llms.txt
        </Link>{' '}
        indexes the docs and{' '}
        <Link className="underline" href="/llms-full.txt">
          /llms-full.txt
        </Link>{' '}
        is all of them in one file, ready to paste into a rules file.
      </p>
    ),
  },
];

export function Checklist() {
  const [done, setDone] = useState<Record<string, boolean>>({});
  useEffect(() => {
    try {
      setDone(JSON.parse(localStorage.getItem(key) ?? '{}'));
    } catch {}
  }, []);
  const toggle = (id: string) => {
    const next = { ...done, [id]: !done[id] };
    setDone(next);
    localStorage.setItem(key, JSON.stringify(next));
  };
  const count = steps.filter((s) => done[s.id]).length;
  return (
    <>
      <p className="mt-4 text-sm text-muted-foreground" aria-live="polite">
        {count} of {steps.length} done
      </p>
      <ol className="mt-6 space-y-4">
        {steps.map((s, i) => (
          <li key={s.id} className="rounded-xl border p-4">
            <div className="flex items-start gap-3">
              <button
                type="button"
                onClick={() => toggle(s.id)}
                aria-pressed={!!done[s.id]}
                aria-label={`Mark "${s.title}" ${done[s.id] ? 'not done' : 'done'}`}
                className={cn(
                  'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded border transition-colors',
                  done[s.id] && 'border-primary bg-primary text-primary-foreground',
                )}
              >
                {done[s.id] && <Check className="size-3.5" />}
              </button>
              <div className="min-w-0 flex-1">
                <h2 className={cn('font-semibold', done[s.id] && 'text-muted-foreground line-through')}>
                  {i + 1}. {s.title}
                </h2>
                <div className="mt-1 text-sm text-muted-foreground">{s.body}</div>
              </div>
            </div>
          </li>
        ))}
      </ol>
    </>
  );
}
