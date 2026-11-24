"""Fail on spellings spec/GLOSSARY.md's "Never" column bans, in what the tools
say: every command's `--help`, every `cpl explain` entry, every diagnostic
message in the registry (`crates/cpl-diag/src/msgs/`), the errors `cpl check`
reports over the corpus and the conformance reject cases, every std doc
comment (what a hover shows), `cpl explain` on conformance files, and the
rest of what `cpl lsp` hovers show over them.

The banned spellings are read from the glossary, not listed here: a plain
spelling matches with its inflections, ignoring case; a `code` spelling
matches exactly; `(except ...)` allows the phrase it names, and the std module
it names (`the `interval` module`) may use the spelling in its docs.

Prose only: words inside backticks or code are quoted, not used.
Enforces handbook/guidelines/errors-checklist.md#r5-name-what-the-user-wrote-in-the-specs-words (errors/R5)
"""
from __future__ import annotations

import json
import os
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor

import corpus
import specgate
from specgate import Problem

HELP = "cpl's help, explain, errors and LSP hovers use no spelling spec/GLOSSARY.md bans"


def split_top(cell):
    """`cell` split at `,` and `;` outside parentheses and backticks."""
    out, depth, code, cur = [], 0, False, ""
    for ch in cell:
        code ^= ch == "`"
        depth += (ch == "(") - (ch == ")") if not code else 0
        if ch in ",;" and not depth and not code:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    return [s for s in out + [cur.strip()] if s]


def inflections(word):
    """`word` and its regular English inflections, as a regex alternation."""
    forms = {word, word + "s", word + "es", word + "ed", word + "ing"}
    if word.endswith("y"):
        forms.add(word[:-1] + "ies")
    if word.endswith("c"):
        forms |= {word + "ked", word + "king"}
    if word.endswith("ed"):
        forms |= {word[:-2], word[:-2] + "s", word[:-2] + "ing"}
    return "(?:" + "|".join(sorted(map(re.escape, forms), key=len, reverse=True)) + ")"


def spec_prose(root):
    """The prose of every spec/ page but the glossary, code blanked."""
    return prose("\n".join(p.read_text() for p in sorted((root / "spec").rglob("*.md")) if p.name != "GLOSSARY.md"))


def banned(glossary=None, spec=None):
    """(compiled pattern, the spelling, the glossary's term, the std modules
    whose docs may use it) for every spelling the glossary's Never column bans,
    read from spec/GLOSSARY.md.

    A spelling banned only in a sense (`index (as a type name)`) or a `code`
    spelling is left out when the spec's own prose uses it as a word: there it
    is another sense or English (`none of them`), which words alone can't tell
    apart. Plain spellings are banned everywhere."""
    root = specgate.ROOT
    glossary = glossary or (root / "spec" / "GLOSSARY.md").read_text()
    spec = spec if spec is not None else spec_prose(root)
    out = []
    for line in glossary.split("\n"):
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if not line.startswith("|") or len(cells) != 3 or cells[0] == "Term" or set(cells[0]) <= set("-: "):
            continue
        term, never = cells[0], cells[2]
        for item in split_top(never):
            m = re.fullmatch(r"(`[^`]+`|[^(]+)(.*)", item)
            spelling, qual = m.group(1).strip(), m.group(2).strip().strip("()")
            if spelling.startswith("`"):
                code = spelling.strip("`")
                if not re.search(r"\w", code) or qual:
                    continue  # punctuation or a code shape: prose can't spell it
                pat = re.compile(r"(?<![\w.])" + re.escape(code) + r"(?![\w(])")
            else:
                words = spelling.split()
                body = r"\s+".join([*map(re.escape, words[:-1]), inflections(words[-1])])
                allow = ""
                said = prose(qual).split()
                if said[:1] == ["except"] and spelling in said:
                    tail = said[said.index(spelling) + 1:]
                    allow = r"(?!\s+" + re.escape(tail[0].rstrip("s")) + ")" if tail else ""
                pat = re.compile(r"(?<![\w-])" + body + r"(?![\w-])" + allow, re.I)
            sensed = qual and not qual.startswith("except")
            if (sensed or spelling.startswith("`")) and re.search(pat.pattern, spec, re.I):
                continue
            out.append((pat, spelling, term, set(re.findall(r"`(\w+)` module", qual))))
    return out


def prose(text):
    """`text` with code spans, fenced blocks and `\\(..\\)` math blanked."""
    text = re.sub(r"```.*?```", " ", text, flags=re.S)
    text = re.sub(r"`[^`\n]*`", " ", text)
    return re.sub(r"\\\(.*?\\\)", " ", text)


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, timeout=300, **kw)


class Lsp:
    def __init__(self, cpl):
        self.p = subprocess.Popen([cpl, "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
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
            if not n:
                raise EOFError(f"cpl lsp exited with {self.p.wait()} on {method}")
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


def registry_messages(root):
    """(where, text) for every row of the diagnostic message registry."""
    row = re.compile(r'^\s*(\w+)\s*=\s*(?:\w+|"[A-Z]\d+")\s*,\s*"((?:[^"\\]|\\.)*)"\s*;', re.M | re.S)
    for f in sorted((root / "crates/cpl-diag/src/msgs").glob("*.rs")):
        text = f.read_text()
        for m in row.finditer(text):
            yield f"{f.relative_to(root)}:{text.count(chr(10), 0, m.start()) + 1} {m.group(1)}", m.group(2)


def std_docs(root):
    """(where, text) for every doc comment line in std/, which hovers show."""
    for f in sorted((root / "std").rglob("*.cp")):
        for i, line in enumerate(f.read_text().split("\n"), 1):
            if m := re.match(r"\s*//[/!] ?(.*)", line):
                yield f"{f.relative_to(root)}:{i}", m.group(1)


def strings(v):
    """Every string in a diagnostic's JSON, but the code it quotes back (fixes, witnesses)."""
    if isinstance(v, str):
        yield v
    elif isinstance(v, list):
        for x in v:
            yield from strings(x)
    elif isinstance(v, dict):
        for k, x in v.items():
            if k not in ("code", "severity", "page", "path", "fixes", "witness"):
                yield from strings(x)


def check_errors(cpl, files):
    """(where, text) for the diagnostics `cpl check --json` reports on each
    file, and a problem for each file it crashes on."""
    def one(f):
        r = sh([cpl, "check", "--json", str(f)])
        try:
            return f, json.loads(r.stdout)["files"], None
        except (ValueError, KeyError):
            return f, [], (r.stderr.strip().splitlines() or [f"exit {r.returncode}"])[-1]
    texts, crashed = [], []
    with ThreadPoolExecutor(os.cpu_count() or 2) as pool:
        for f, out, err in pool.map(one, files):
            rel = f.relative_to(specgate.ROOT)
            if err:
                crashed.append(Problem(f"cpl check {rel}", f"crashed, so its errors went unread: {err}"))
            for d in (d for fo in out for d in fo["diagnostics"]):
                texts.extend((f"cpl check {rel}:{d['start']['line']} {d['code']}", s) for s in strings(d))
    return texts, crashed


def error_files():
    """Every corpus port, dream, spec and perf program, and every conformance case that expects a diagnostic."""
    files = [p for pkg in corpus.problems() for p in (pkg.port, pkg.dream, pkg.spec, pkg.dir / "perf.cp") if p and p.is_file()]
    return files + [c for c in specgate.cases() if re.match(r"// expect: [EW]\d", specgate.read(c))]


def args(ap) -> None:
    ap.add_argument("--cpl")
    ap.add_argument("--files", type=int, default=200, help="conformance files to explain and hover")


def run(args) -> list[Problem]:
    cpl = args.cpl or str(specgate.ROOT / "target" / "debug" / "cpl")
    texts, problems = [], []  # (where, text)
    top = sh([cpl, "--help"]).stdout
    texts.append(("cpl --help", top))
    for cmd in re.findall(r"^  (\w+)\b", top, re.M):
        r = sh([cpl, cmd, "--help"])
        texts.append((f"cpl {cmd} --help", r.stdout + r.stderr))
    for e in json.loads(sh([cpl, "explain", "--json"]).stdout):
        for field in ("title", "summary", "explanation", "fix"):
            texts.append((f"cpl explain {e['code']} ({field})", e.get(field) or ""))
    texts.extend(registry_messages(specgate.ROOT))
    errors, crashed = check_errors(cpl, error_files())
    texts.extend(errors)
    problems.extend(crashed)
    docs = list(std_docs(specgate.ROOT))
    texts.extend(docs)
    quoted = {t.strip() for _, t in docs}
    cases = specgate.cases()
    step = max(1, len(cases) // args.files)
    lsp = Lsp(cpl)
    for case in cases[::step][: args.files]:
        rel = case.relative_to(specgate.ROOT)
        texts.append((f"cpl explain {rel}", sh([cpl, "explain", str(case)]).stdout))
        try:
            for h in lsp.hovers(case, case.read_text()):
                texts.append((f"hover in {rel}", "\n".join(l for l in h.split("\n") if l.startswith("```") or l.strip() not in quoted)))
        except EOFError as e:
            problems.append(Problem(f"hover in {rel}", f"{e}, so its hovers went unread"))
            lsp = Lsp(cpl)
    lsp.p.kill()
    bad = {}
    rules = banned()
    for where, text in texts:
        plain = prose(text)
        for pat, spelling, term, modules in rules:
            if any(where.startswith(f"std/{m}.cp:") for m in modules):
                continue
            for m in pat.finditer(plain):
                ctx = plain[max(0, m.start() - 40) : m.end() + 30].replace("\n", " ")
                bad.setdefault((m.group().lower(), term, " ".join(ctx.split())), where)
    return problems + [Problem(where, f"`{word}`, the spec says {term}: ...{ctx}...")
                       for (word, term, ctx), where in sorted(bad.items(), key=lambda kv: kv[1])]
