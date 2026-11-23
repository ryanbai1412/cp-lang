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
          The extension is on the edge release; Marketplace and Open VSX (Cursor, Windsurf, VSCodium) installs <Soon />.
          On Windows, run the same two lines in PowerShell.
        </p>
        <Code>
          {
            'curl.exe -fLO https://github.com/ryanbai1412/cp-lang/releases/download/edge/cp-lang.vsix\ncode --install-extension cp-lang.vsix'
          }
        </Code>
      </>
    ),
  },
  {
    id: 'neovim',
    name: 'Neovim',
    body: (
      <>
        <p>
          Neovim 0.10 or later. One command installs the plugin as a package, with nothing to add to your config: every{' '}
          <code>.cp</code> buffer gets highlighting and <code>cpl lsp</code> (diagnostics, completion, hover, go to
          definition with <code>vim.lsp.buf.definition()</code>, formatting with <code>vim.lsp.buf.format()</code>), and{' '}
          <code>:Cpl test</code>, <code>:Cpl stress</code> and <code>:Cpl submit</code> run on the current file.
        </p>
        <p className="mt-3">Linux and macOS:</p>
        <Code>
          {
            'mkdir -p ~/.local/share/nvim/site/pack/cpl/start\ncurl -fsSL https://github.com/ryanbai1412/cp-lang/releases/download/edge/editors.tar.gz |\n  tar -xzf - --strip-components=1 -C ~/.local/share/nvim/site/pack/cpl/start cp-lang-editors/neovim'
          }
        </Code>
        <p className="mt-3">Windows (PowerShell):</p>
        <Code>
          {
            '$d = "$env:LOCALAPPDATA\\nvim-data\\site\\pack\\cpl\\start"; mkdir -Force $d\ncurl.exe -fsSLo "$env:TEMP\\cpl-editors.tar.gz" https://github.com/ryanbai1412/cp-lang/releases/download/edge/editors.tar.gz\ntar -xzf "$env:TEMP\\cpl-editors.tar.gz" --strip-components=1 -C $d cp-lang-editors/neovim'
          }
        </Code>
        <p className="mt-3">Run it again to update. With lazy.nvim, point a spec at the same folder instead.</p>
      </>
    ),
  },
  {
    id: 'vim',
    name: 'Vim',
    body: (
      <>
        <p>
          The same package gives Vim 8.2 or later the filetype and highlighting. On Windows, use <code>~/vimfiles</code>{' '}
          for <code>~/.vim</code> and the PowerShell lines from the Neovim tab.
        </p>
        <Code>
          {
            'mkdir -p ~/.vim/pack/cpl/start\ncurl -fsSL https://github.com/ryanbai1412/cp-lang/releases/download/edge/editors.tar.gz |\n  tar -xzf - --strip-components=1 -C ~/.vim/pack/cpl/start cp-lang-editors/neovim'
          }
        </Code>
        <p className="mt-3">
          For diagnostics and completion, install{' '}
          <a className="underline" href="https://github.com/prabirshrestha/vim-lsp">
            vim-lsp
          </a>{' '}
          and add this to your <code>.vimrc</code>:
        </p>
        <Code>
          {"au User lsp_setup call lsp#register_server({'name': 'cpl', 'cmd': ['cpl', 'lsp'], 'allowlist': ['cp']})"}
        </Code>
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
      <>
        <p>
          IntelliJ IDEA Ultimate, CLion, PyCharm Professional or another commercial JetBrains IDE, 2024.3 or later: the
          free Community editions lack the language server API the plugin uses. Marketplace install <Soon />. Download
          the plugin from the edge release (on Windows, in PowerShell):
        </p>
        <Code>
          {'curl.exe -fLO https://github.com/ryanbai1412/cp-lang/releases/download/edge/cp-lang-jetbrains.zip'}
        </Code>
        <p className="mt-3">
          Then Settings → Plugins → ⚙ → Install Plugin from Disk, pick the zip as it is, and restart. A <code>.cp</code>{' '}
          file gets diagnostics, completion, go to definition and formatting from <code>cpl lsp</code>; set the path to{' '}
          <code>cpl</code> under Settings → Tools → CP Lang if it is not on the <code>PATH</code>.
        </p>
      </>
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
          One line installs a prebuilt binary of the edge build, the newest commit of <code>main</code> that passed
          every check.
        </p>
        <p className="mt-3">Linux and macOS:</p>
        <Code>{'curl -fsSL https://cp-lang.org/install.sh | sh -s -- --channel edge'}</Code>
        <p className="mt-3">Windows (PowerShell):</p>
        <Code>{'& ([scriptblock]::Create((irm https://cp-lang.org/install.ps1))) -Channel edge'}</Code>
        <p className="mt-3">
          Then open a new terminal and run <code>cpl --version</code>. <code>cpl self update</code> fetches newer builds
          later. On a platform without a binary, build it with Rust:
        </p>
        <Code>{'cargo install --locked --git https://github.com/ryanbai1412/cp-lang cpl-cli\ncpl --version'}</Code>
      </>
    ),
  },
  {
    id: 'editor',
    title: 'Install the editor extension',
    body: (
      <>
        <p>
          Every editor talks to the same language server, <code>cpl lsp</code>: diagnostics as you type, hover,
          completion, go to definition, rename and quick fixes.
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
        <p>
          Create a problem, keep its tests running on every save, stress it against a brute, then check it&apos;s ready.
        </p>
        <Code>{'cpl new a\ncpl watch a\ncpl stress a.cp\ncpl submit a.cp'}</Code>
        <p className="mt-2">
          To import a problem with its samples and limits instead, install the{' '}
          <a className="underline" href="https://github.com/jmerle/competitive-companion">
            Competitive Companion
          </a>{' '}
          browser extension, run <code>cpl new --listen</code>, and click the extension&apos;s green plus on the problem
          page (or on the contest page, for every problem). <code>cpl run a.cp</code> then runs every sample.
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
