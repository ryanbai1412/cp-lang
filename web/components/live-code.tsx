"use client";

import { Loader2, Pencil, Play } from "lucide-react";
import { type ReactNode, useState } from "react";
import { CodeEditor } from "@/components/editor/code-editor";
import { encodeShare, formatDiagnostics } from "@/components/runner";
import { Button } from "@/components/ui/button";
import { cpl } from "@/lib/cpl";
import { cn } from "@/lib/utils";

// A guide's highlighted block plus Run (when the gate ran it with its input) and a playground link.
// The compiler loads on the first Run, so a page of these fetches no WASM until then.
// Edit swaps the block for an editor with diagnostics, hover and completion.
export function LiveCode({
  source: original,
  stdin = "",
  ran,
  children,
}: {
  source: string;
  stdin?: string;
  ran?: string;
  children: ReactNode;
}) {
  const [source, setSource] = useState(original);
  const [editing, setEditing] = useState(false);
  const [out, setOut] = useState<
    { text: string; ok: boolean } | "running" | null
  >(null);
  const run = async () => {
    setOut("running");
    try {
      const r = await cpl.run(source, stdin);
      const errors = r.diagnostics.length
        ? formatDiagnostics(r.diagnostics, source)
        : r.stderr;
      setOut({
        text: r.stdout + (errors ? (r.stdout ? "\n" : "") + errors : ""),
        ok: r.ok,
      });
    } catch (e) {
      setOut({ text: (e as Error).message, ok: false });
    }
  };
  return (
    <div className="live-code my-4 [&>figure]:my-0">
      {editing ? (
        <div className="not-prose rounded-md border">
          <CodeEditor value={source} onChange={setSource} onRun={run} />
        </div>
      ) : (
        children
      )}
      <div className="not-prose mt-1.5 flex flex-wrap items-center gap-2 text-xs">
        {ran && (
          <Button
            size="sm"
            variant="outline"
            className="h-7"
            onClick={run}
            disabled={out === "running"}
          >
            {out === "running" ? (
              <Loader2 className="animate-spin" />
            ) : (
              <Play />
            )}
            Run
          </Button>
        )}
        {!editing && (
          <Button
            size="sm"
            variant="ghost"
            className="h-7 text-muted-foreground"
            onClick={() => setEditing(true)}
          >
            <Pencil />
            Edit
          </Button>
        )}
        <Button
          size="sm"
          variant="ghost"
          className="h-7 text-muted-foreground"
          asChild
        >
          <a
            href={`/play#${typeof window === "undefined" ? "" : encodeShare(source, stdin)}`}
            suppressHydrationWarning
          >
            Open in playground
          </a>
        </Button>
        {stdin && (
          <span className="text-muted-foreground">
            input: {stdin.trim().split("\n").join(" / ")}
          </span>
        )}
      </div>
      {out && (
        <pre tabIndex={0}
          role="status"
          className={cn(
            "not-prose mt-1.5 max-h-64 overflow-auto rounded-md bg-muted p-3 font-mono text-xs whitespace-pre-wrap",
            out !== "running" && !out.ok && "text-destructive",
          )}
        >
          {out === "running"
            ? "Loading the compiler and running…"
            : out.text || "(no output)"}
        </pre>
      )}
    </div>
  );
}
