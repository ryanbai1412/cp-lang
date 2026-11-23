"use client";

import { Check, Download } from "lucide-react";
import Link from "next/link";
import { type ReactNode, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import published from "@/generated/downloads.json";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { channelNotes } from "@/components/install";
import facts from "@/lib/facts.json";
import { cn } from "@/lib/utils";

const key = "cpl-setup";

function Code({ children }: { children: string }) {
  return (
    <pre className="mt-2 overflow-x-auto rounded-md border bg-muted/40 p-3 text-xs">
      {children}
    </pre>
  );
}

type Asset = { url: string; size: number } | undefined;
const downloads = published as unknown as Record<
  string,
  Record<"vscode" | "jetbrains", Asset>
>;

// One button per release channel that ships the file, from the releases' own latest.json.
function Downloads({
  asset,
  label,
}: {
  asset: "vscode" | "jetbrains";
  label: string;
}) {
  const shipped = Object.keys(downloads).filter((c) => downloads[c][asset]);
  return (
    <div className="mt-2 flex flex-wrap gap-2">
      {shipped.map((c) => {
        const file = downloads[c][asset]!;
        return (
          <a
            key={c}
            href={file.url}
            className="inline-flex items-center gap-1.5 rounded-md border bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:opacity-90"
          >
            <Download className="size-3.5" />
            {label} ({c}, {Math.round(file.size / 1024)} KB)
          </a>
        );
      })}
    </div>
  );
}

function Soon() {
  return (
    <Badge variant="outline" className="ml-2 align-middle text-[10px]">
      not published yet
    </Badge>
  );
}

// The one-step install `cpl setup <editor>` does, shown above each tab's manual steps.
function SetupCmd({ editor }: { editor: string }) {
  return (
    <>
      <p>
        With <code>cpl</code> installed, one command sets it up on Linux, macOS
        and Windows (run it again after <code>cpl self update</code>):
      </p>
      <Code>{`cpl setup ${editor}`}</Code>
      <p className="mt-3">Or by hand:</p>
    </>
  );
}

const editors: { id: string; name: string; body: ReactNode }[] = [
  {
    id: "vscode",
    name: "VS Code",
    body: (
      <>
        <SetupCmd editor="vscode" />
        <p>
          Download the extension, then in VS Code (or Cursor, Windsurf,
          VSCodium) open Extensions → ⋯ → Install from VSIX… and pick the file.
          Marketplace and Open VSX installs <Soon />.
        </p>
        <p className="mt-3">
          In a folder VS Code hasn&apos;t been told to trust (Restricted Mode),
          a <code>.cp</code> file is highlighted but gets no diagnostics,
          completion or runs: trust the folder and the server starts.
        </p>
        <Downloads asset="vscode" label="Download cp-lang.vsix" />
        <p className="mt-3">
          Or from a terminal (on Windows, the same two lines in PowerShell):
        </p>
        <Code>
          {
            "curl.exe -fLO https://github.com/ryanbai1412/cp-lang/releases/download/edge/cp-lang.vsix\ncode --install-extension cp-lang.vsix"
          }
        </Code>
      </>
    ),
  },
  {
    id: "neovim",
    name: "Neovim",
    body: (
      <>
        <SetupCmd editor="neovim" />
        <p>
          Neovim 0.10 or later (Ubuntu 22.04&apos;s apt has 0.6: take the
          AppImage or tarball from{" "}
          <a href="https://github.com/neovim/neovim/releases">
            Neovim&apos;s releases
          </a>
          ). One command installs the plugin as a package, with nothing to add
          to your config: every <code>.cp</code> buffer gets highlighting and{" "}
          <code>cpl lsp</code> (diagnostics, completion, hover, go to definition
          with <code>vim.lsp.buf.definition()</code>, formatting with{" "}
          <code>vim.lsp.buf.format()</code>), and <code>:Cpl test</code>,{" "}
          <code>:Cpl stress</code> and <code>:Cpl submit</code> run on the
          current file.
        </p>
        <p className="mt-3">Linux and macOS:</p>
        <Code>
          {
            "mkdir -p ~/.local/share/nvim/site/pack/cpl/start\ncurl -fsSL https://github.com/ryanbai1412/cp-lang/releases/download/edge/editors.tar.gz |\n  tar -xzf - --strip-components=1 -C ~/.local/share/nvim/site/pack/cpl/start cp-lang-editors/neovim"
          }
        </Code>
        <p className="mt-3">Windows (PowerShell):</p>
        <Code>
          {
            '$d = "$env:LOCALAPPDATA\\nvim-data\\site\\pack\\cpl\\start"; mkdir -Force $d\ncurl.exe -fsSLo "$env:TEMP\\cpl-editors.tar.gz" https://github.com/ryanbai1412/cp-lang/releases/download/edge/editors.tar.gz\ntar -xzf "$env:TEMP\\cpl-editors.tar.gz" --strip-components=1 -C $d cp-lang-editors/neovim'
          }
        </Code>
        <p className="mt-3">
          Run it again to update. With lazy.nvim, point a spec at the same
          folder instead.
        </p>
      </>
    ),
  },
  {
    id: "vim",
    name: "Vim",
    body: (
      <>
        <SetupCmd editor="vim" />
        <p>
          That gives Vim 8.2 or later the filetype, highlighting and indent,
          and installs{" "}
          <a
            className="underline"
            href="https://github.com/prabirshrestha/vim-lsp"
          >
            vim-lsp
          </a>{" "}
          beside it, through which <code>cpl lsp</code> gives live
          diagnostics, completion, hover and go to definition. If vim-lsp or
          yegappan/lsp is already installed, it uses that one instead. Without
          any LSP client, <code>:make</code> runs <code>cpl check</code> and
          puts each error in the quickfix list (<code>:copen</code>), and{" "}
          <code>gq</code> formats through <code>cpl fmt</code>. On Windows,
          use <code>~/vimfiles</code> for <code>~/.vim</code>.
        </p>
        <p className="mt-3">Without <code>cpl setup</code>, by hand:</p>
        <Code>
          {
            "mkdir -p ~/.vim/pack/cpl/start\ncurl -fsSL https://github.com/ryanbai1412/cp-lang/releases/download/edge/editors.tar.gz |\n  tar -xzf - --strip-components=1 -C ~/.vim/pack/cpl/start cp-lang-editors/neovim\ngit clone https://github.com/prabirshrestha/vim-lsp ~/.vim/pack/cpl/start/vim-lsp"
          }
        </Code>
      </>
    ),
  },
  {
    id: "helix",
    name: "Helix",
    body: (
      <>
        <p>
          Append <code>editors/helix/languages.toml</code> to{" "}
          <code>~/.config/helix/languages.toml</code>, then:
        </p>
        <Code>
          {
            "mkdir -p ~/.config/helix/runtime/queries\nln -s /path/to/cp-lang/editors/helix/runtime/queries/cp ~/.config/helix/runtime/queries/cp\nhx --grammar build\nhx --health cp"
          }
        </Code>
      </>
    ),
  },
  {
    id: "zed",
    name: "Zed",
    body: (
      <p>
        Registry install <Soon />. Until then, run{" "}
        <code>zed: install dev extension</code> and pick{" "}
        <code>editors/zed</code> (needs Rust with the <code>wasm32-wasip1</code>{" "}
        target).
      </p>
    ),
  },
  {
    id: "emacs",
    name: "Emacs",
    body: (
      <Code>
        {
          "(add-to-list 'load-path \"/path/to/cp-lang/editors/emacs\")\n(require 'cp-mode)\n(add-hook 'cp-mode-hook #'eglot-ensure)"
        }
      </Code>
    ),
  },
  {
    id: "jetbrains",
    name: "JetBrains",
    body: (
      <>
        <SetupCmd editor="jetbrains" />
        <p>
          Any JetBrains IDE, 2024.3 or later. The commercial ones run{" "}
          <code>cpl lsp</code> through their own language server client; free
          IntelliJ IDEA and the Community editions run it through the LSP4IJ
          plugin, which <code>cpl setup jetbrains</code> installs too.
          Marketplace install <Soon />.
        </p>
        <Downloads asset="jetbrains" label="Download cp-lang-jetbrains.zip" />
        <p className="mt-3">Or from a terminal (on Windows, in PowerShell):</p>
        <Code>
          {
            "curl.exe -fLO https://github.com/ryanbai1412/cp-lang/releases/download/edge/cp-lang-jetbrains.zip"
          }
        </Code>
        <p className="mt-3">
          Then Settings → Plugins → ⚙ → Install Plugin from Disk, pick the zip
          as it is, and restart. A <code>.cp</code> file gets diagnostics,
          completion, go to definition and formatting from <code>cpl lsp</code>;
          set the path to <code>cpl</code> under Settings → Tools → CP Lang if
          it is not on the <code>PATH</code>. Open the folder as a project:
          a file opened on its own goes to LightEdit mode, which has no
          language server, and an untrusted project opens in Safe Mode until
          you choose Trust Project.
        </p>
      </>
    ),
  },
];

const steps: { id: string; title: string; body: ReactNode }[] = [
  {
    id: "install",
    title: "Install cpl",
    body: (
      <>
        <p>
          One line installs a prebuilt binary. Pick a channel:
        </p>
        <Tabs defaultValue="edge" className="mt-3">
          <TabsList>
            <TabsTrigger value="edge">Edge</TabsTrigger>
            <TabsTrigger value="stable">
              Stable
              <Badge variant="outline" className="ml-2 text-[10px]">beta</Badge>
            </TabsTrigger>
          </TabsList>
          {(["edge", "stable"] as const).map((ch) => (
            <TabsContent key={ch} value={ch}>
              <p className="mt-2">
                {ch === "edge" ? "Edge is " : "Stable is "}
                {channelNotes[ch]}.
              </p>
              <p className="mt-3">Linux and macOS:</p>
              <Code>{facts.install[ch].sh}</Code>
              <p className="mt-3">Windows (PowerShell):</p>
              <Code>{facts.install[ch].ps1}</Code>
            </TabsContent>
          ))}
        </Tabs>
        <p className="mt-3">
          Then open a new terminal and run <code>cpl --version</code>: edge
          builds print <code>{facts.edgeVersion}</code>, the version
          these docs describe, and an older <code>0.2.x</code> install needs{" "}
          <code>cpl self update</code>, which also fetches newer builds later.
          On a platform without a binary, build it with Rust:
        </p>
        <Code>
          {`${facts.install.cargo}\ncpl --version`}
        </Code>
      </>
    ),
  },
  {
    id: "editor",
    title: "Install the editor extension",
    body: (
      <>
        <p>
          Every editor talks to the same language server, <code>cpl lsp</code>:
          diagnostics as you type, hover, completion, go to definition, rename
          and quick fixes.
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
            <TabsContent
              key={e.id}
              value={e.id}
              className="text-sm text-muted-foreground"
            >
              {e.body}
            </TabsContent>
          ))}
        </Tabs>
      </>
    ),
  },
  {
    id: "verify",
    title: "Check it works",
    body: (
      <>
        <p>
          Open a <code>.cp</code> file and type <code>let x = </code>; an error
          should appear as you type. Then:
        </p>
        <Code>{"echo 'print(\"hi\")' > hi.cp\ncpl run hi.cp"}</Code>
      </>
    ),
  },
  {
    id: "judge",
    title: "Pick your judge",
    body: (
      <>
        <p>
          A judge profile fixes the compiler, flags, stack and source limit, so
          local runs behave like the judge. See{" "}
          <Link className="underline" href="/tools/judges">
            all profiles
          </Link>
          .
        </p>
        <Code>{"cpl judges\ncpl build --judge codeforces a.cp"}</Code>
      </>
    ),
  },
  {
    id: "loop",
    title: "Set up the contest loop",
    body: (
      <>
        <p>
          Create a problem, keep its tests running on every save, stress it
          against a brute, then check it&apos;s ready.
        </p>
        <Code>
          {"cpl new a\ncpl watch a\ncpl stress a.cp\ncpl submit a.cp"}
        </Code>
        <p className="mt-2">
          To import a problem with its samples and limits instead, install the{" "}
          <a
            className="underline"
            href="https://github.com/jmerle/competitive-companion"
          >
            Competitive Companion
          </a>{" "}
          browser extension, run <code>cpl new --listen</code>, and click the
          extension&apos;s green plus on the problem page (or on the contest
          page, for every problem). If Chrome hides the icon, pin it from the
          puzzle-piece menu, or press its shortcut, Ctrl+Shift+U (⌘⇧U on macOS). <code>cpl run a.cp</code> then runs every
          sample.
        </p>
      </>
    ),
  },
  {
    id: "ai",
    title: "Optional: point your AI assistant at the docs",
    body: (
      <p>
        <Link className="underline" href="/llms.txt">
          /llms.txt
        </Link>{" "}
        indexes the docs and{" "}
        <Link className="underline" href="/llms-full.txt">
          /llms-full.txt
        </Link>{" "}
        is all of them in one file, ready to paste into a rules file.
      </p>
    ),
  },
];

export function Checklist() {
  const [done, setDone] = useState<Record<string, boolean>>({});
  useEffect(() => {
    try {
      setDone(JSON.parse(localStorage.getItem(key) ?? "{}"));
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
                aria-label={`Mark "${s.title}" ${done[s.id] ? "not done" : "done"}`}
                className={cn(
                  "mt-0.5 flex size-5 shrink-0 items-center justify-center rounded border transition-colors",
                  done[s.id] &&
                    "border-primary bg-primary text-primary-foreground",
                )}
              >
                {done[s.id] && <Check className="size-3.5" />}
              </button>
              <div className="min-w-0 flex-1">
                <h2
                  className={cn(
                    "font-semibold",
                    done[s.id] && "text-muted-foreground line-through",
                  )}
                >
                  {i + 1}. {s.title}
                </h2>
                <div className="mt-1 text-sm text-muted-foreground">
                  {s.body}
                </div>
              </div>
            </div>
          </li>
        ))}
      </ol>
    </>
  );
}
