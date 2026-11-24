#!/usr/bin/env python3
# Enforces handbook/guidelines/errors-checklist.md#r5-name-what-the-user-wrote-in-the-specs-words (errors/R5)
"""Fail on spellings spec/GLOSSARY.md's "Never" column bans, in what the tools
say: every command's `--help`, every `cpl explain` entry, `cpl explain` on
conformance files, and `cpl lsp` hovers over them.

Prose only: words inside backticks or code are quoted, not used. Usage:
    uv run scripts/tool-vocab.py [--cpl target/debug/cpl] [--files 200]
"""
import argparse, json, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NEVER = [
    (r"\bindex (space )?types?\b", "domain"),
    (r"\bnode ids?\b", "position"),
    (r"\bfenceposts?\b", "cut"),
    (r"\bborrow(ed|s|ing)?\b", "lent"),
    (r"\binout\b", "`mut`"),
    # `template functor` is the C++ a closure argument lowers to (spec/core/13).
    (r"(?<!template )\bfunctors?\b|\bop objects?\b", "operation value"),
    (r"\baxioms?\b", "`law`"),
    (r"\bMint\b|\bModInt\b", "`Mod<P>`"),
    (r"\bdebug builds?\b", "checked build"),
    (r"\bpanic(s|ked)?\b", "trap"),
    (r"\bunimplemented\b", "`todo`"),
    (r"\bhalf-edges?\b|\badjacency entr(y|ies)\b", "arc"),
    (r"\bnil\b", "`None`"),
]


def prose(text):
    """`text` with code spans, fenced blocks and `\\(..\\)` math blanked."""
    text = re.sub(r"```.*?```", " ", text, flags=re.S)
    text = re.sub(r"`[^`\n]*`", " ", text)
    return re.sub(r"\\\(.*?\\\)", " ", text)


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, timeout=120, **kw)


class Lsp:
    def __init__(self, cpl):
        self.p = subprocess.Popen([cpl, "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.id = 0
        self.ask("initialize", {"processId": None, "rootUri": None, "capabilities": {}})
        self.say("initialized", {})

    def say(self, method, params, id=None):
        msg = {"jsonrpc": "2.0", "method": method, "params": params}
        if id is not None:
            msg["id"] = id
        body = json.dumps(msg).encode()
        self.p.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.p.stdin.flush()

    def ask(self, method, params):
        self.id += 1
        self.say(method, params, self.id)
        while True:
            n = 0
            while (line := self.p.stdout.readline().strip()):
                if line.lower().startswith(b"content-length:"):
                    n = int(line.split(b":")[1])
            reply = json.loads(self.p.stdout.read(n))
            if reply.get("id") == self.id and "method" not in reply:
                return reply.get("result")

    def hovers(self, path, src):
        uri = path.resolve().as_uri()
        self.say("textDocument/didOpen", {"textDocument": {"uri": uri, "languageId": "cp", "version": 1, "text": src}})
        seen = set()
        for i, line in enumerate(src.split("\n")):
            if line.lstrip().startswith("//"):
                continue
            for m in re.finditer(r"\b[A-Za-z_]\w*", line):
                if m.group() in seen:
                    continue
                seen.add(m.group())
                pos = {"line": i, "character": m.start()}
                r = self.ask("textDocument/hover", {"textDocument": {"uri": uri}, "position": pos})
                if r and isinstance(r.get("contents"), dict):
                    yield r["contents"].get("value", "")
        self.say("textDocument/didClose", {"textDocument": {"uri": uri}})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpl", default=str(ROOT / "target" / "debug" / "cpl"))
    ap.add_argument("--files", type=int, default=200, help="conformance files to explain and hover")
    args = ap.parse_args()
    cpl = args.cpl
    texts = []  # (where, text)
    top = run([cpl, "--help"]).stdout
    texts.append(("cpl --help", top))
    for cmd in re.findall(r"^  (\w+)\b", top, re.M):
        r = run([cpl, cmd, "--help"])
        texts.append((f"cpl {cmd} --help", r.stdout + r.stderr))
    for e in json.loads(run([cpl, "explain", "--json"]).stdout):
        for field in ("title", "summary", "explanation", "fix"):
            texts.append((f"cpl explain {e['code']} ({field})", e.get(field) or ""))
    cases = sorted(p for p in (ROOT / "spec" / "conformance").rglob("*.cp"))
    step = max(1, len(cases) // args.files)
    lsp = Lsp(cpl)
    hovered = 0
    for case in cases[::step][: args.files]:
        rel = case.relative_to(ROOT)
        texts.append((f"cpl explain {rel}", run([cpl, "explain", str(case)]).stdout))
        for h in lsp.hovers(case, case.read_text()):
            hovered += 1
            texts.append((f"hover in {rel}", h))
    lsp.p.kill()
    bad = {}
    for where, text in texts:
        plain = prose(text)
        for pat, term in NEVER:
            for m in re.finditer(pat, plain, re.I if pat[2:3].islower() else 0):
                ctx = plain[max(0, m.start() - 40) : m.end() + 30].replace("\n", " ")
                bad.setdefault((m.group().lower(), term, ctx.strip()), where)
    for (word, term, ctx), where in sorted(bad.items(), key=lambda kv: kv[1]):
        print(f"{where}: `{word}`, the spec says {term}: ...{ctx}...")
    print(f"tool-vocab: {len(texts)} texts ({hovered} hovers), {len(bad)} banned spelling(s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
