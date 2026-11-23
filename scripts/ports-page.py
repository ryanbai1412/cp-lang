#!/usr/bin/env python3
"""ports-page.py: regenerate research/PORTS.md, the newest ports first.

Each row links the CP Lang program, the accepted C++ it was ported from, the
statement and the friction report, with contestant-token counts against the
C++ (library code stripped) and the taste-review score
(research/reviews/). Dates are when the port was first committed.

  scripts/ports-page.py            write research/PORTS.md
  scripts/ports-page.py --tokens   print the token-ratio summary only

Port discovery and the CP tokenizer come from scripts/review-check.py; the
C++ tokenizer, library stripping and friction parsing are kept here (they
lived in scripts/scoreboard.py before it was replaced by cpl-conform).
"""
import importlib.util, json, os, re, subprocess, sys

_spec = importlib.util.spec_from_file_location(
    "review_check", os.path.join(os.path.dirname(os.path.abspath(__file__)), "review-check.py"))
rc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(rc)

ROOT = rc.ROOT
FRICTIONS_DIR = os.path.join(ROOT, "research", "frictions")
DEFAULT_BASELINE = os.path.join(ROOT, "research", "scoreboard.json")
TIME_P50_GOAL = 1.1
_read_text = rc._read_text
tokenize_cp = rc.tokenize_cp


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


STDLIB_AREAS = (
    "graphs",
    "trees",
    "ranges",
    "strings",
    "math",
    "flows",
    "containers",
    "geometry",
    "dp",
    "seq",
)


_AREA_KEYWORDS = (
    ("flows", ("flow", "max_flow", "min_cost", "dinic", "matching", "bipartite", "hopcroft", "hungarian", "gomory")),
    ("geometry", ("geometry", "convex hull", "polygon", "cross product", "polar", "half-plane", "segment intersect", "point in polygon")),
    ("ranges", ("segtree", "lazysegtree", "fenwick", "sparsetable", "sparse table", "intervalmap", "interval partition", "odt", "range query", "segment tree", "mo's")),
    ("trees", ("hld", "lca", "centroid", "reroot", "euler tour", "permutation tree", "virtual tree", "treap", "link-cut", "tree")),
    ("strings", ("kmp", "z-func", "suffix", "aho", "rolling hash", "string", "`str`", "str ", "s[", "palindrom", "trie")),
    ("graphs", ("dijkstra", "bfs", "dfs", "scc", "2-sat", "twosat", "mst", "kruskal", "dsu", "toposort", "graph", "shortest path", "bridge", "articulation")),
    ("math", ("mod<", "comb<", "ntt", "fft", "convolv", "poly", "matrix", "gauss", "prime", "factor", "gcd", "crt", "binomial", "mint", "xor basis", "linear basis")),
    ("containers", ("sortedbag", "sortedset", "sortedmap", "minheap", "maxheap", "heap", "deque", "bitset", "map<", "set<", "counter", "unordered_map", "priority_queue", "hash map", "multiset")),
    ("dp", ("dp", "knapsack", "aliens", "monge", "cht", "li chao", "sos", "bitmask dp", "memo")),
    ("seq", ("std.seq", "runs", "chunk_by", "rotate", "reverse", "sorted(", "sort(", "dedup", "slice", "step(", "count(", "prod(", "sum(", "windows", "zip", "enumerate", "bsearch", "lower_bound")),
)


_CPP_TOKEN_RE = re.compile(
    r"""
      //[^\n]*                                      # line comment (skip)
    | /\*[\s\S]*?\*/                                # block comment (skip)
    | R"([^(]*)\([\s\S]*?\)\1"                      # raw string literal
    | "(?:\\.|[^"\\])*"                             # string literal
    | '(?:\\.|[^'\\])*'                             # char literal
    | 0[xXbB][0-9a-fA-F']+                          # radix int literal
    | \d[\d']*(?:\.\d[\d']*)?(?:[eE][+-]?\d[\d']*)?[uUlLfF]*
    | [A-Za-z_]\w*                                  # identifier / keyword
    | <=>|<<=|>>=|::|->\*|->|\.\.\.|<<|>>|<=|>=|==|!=|&&|\|\||\+\+|--|\+=|-=|\*=|/=|%=|&=|\|=|\^=
    | [^\s]                                         # single punctuation/operator
    """,
    re.VERBOSE,
)


def tokenize_cpp(src):
    """Return list of lexical tokens in C++ source (no comments/whitespace)."""
    tokens = []
    for m in _CPP_TOKEN_RE.finditer(src):
        tok = m.group(0)
        if tok.startswith("//") or tok.startswith("/*"):
            continue
        tokens.append(tok)
    return tokens


_LIBRARY_NAMES = re.compile(
    r"^(dsu|uf|unionfind|union_find|fenwick|bit|fenw|segtree|seg|segmenttree|lazyseg|lazy|sparsetable|sparse|rmq|lca|"
    r"mint|modint|mod_int|binpow|modpow|mpow|pw|power|modinv|inv|inverse|fact|choose|ncr|nck|binom|comb|sieve|"
    r"chash|custom_hash|hash|hasher|matrix|mat|cht|linecontainer|line|dinic|flow|maxflow|mcmf|hld|treap|trie|"
    r"kmp|zfunc|z_function|manacher|suffixarray|compress|dijkstra|scc|tarjan|fdiv|cdiv|floor_div|ceil_div|"
    r"ckmin|ckmax|chmin|chmax|gcd|lcm|extgcd|exgcd|rng|rnd|rand_int|rev|reversed|bset|ost|min_heap|pq)$",
    re.I,
)


_LIBRARY_WORDS = re.compile(
    r"seg_?tree|segment_?tree|fenwick|sparse_?table|union_?find|disjoint_?set|mod_?int|montgomery|"
    r"suffix_?(array|automaton)|aho_?corasick|li_?chao|link_?cut|centroid|heavy_?light|convex_?hull|"
    r"min_?cost|max_?flow|bipartite_?match|hopcroft|y_?combinator|fast_?(io|input|output|scanner|reader)|"
    r"^(input)?reader$|^kattio$|^fastio$",
    re.I,
)


_TEMPLATE_TYPE = re.compile(r"^template\s*<[^;{]*>\s*(struct|class)\b", re.S)


_IO_SETUP = re.compile(r"sync_with_stdio|cin\.tie|cin\.exceptions|freopen|setprecision")


def _top_level_items(lines):
    """(first, last) 1-based line spans of the top-level items of a C++/Java file."""
    items, depth, start, cont, ifdef = [], 0, None, False, 0
    for i, raw in enumerate(lines, 1):
        s = raw.strip()
        if start is None:
            if not s:
                continue
            start = i
        pre = s.startswith("#") or cont
        cont = pre and s.endswith("\\")
        if pre:
            if re.match(r"#\s*if", s):
                ifdef += 1
            if re.match(r"#\s*endif", s):
                ifdef -= 1
        else:
            code = re.sub(r'"(\\.|[^"\\])*"|\'(\\.|[^\'\\])*\'|//.*', "", s)
            depth += code.count("{") - code.count("}")
        done = (pre and not cont and ifdef <= 0) or (
            not pre and ifdef <= 0 and depth <= 0 and s.endswith((";", "}"))
        )
        if done:
            items.append((start, i))
            start, depth = None, 0
    if start is not None:
        items.append((start, len(lines)))
    return items


def _item_name(text):
    m = re.search(r"\b(?:struct|class|namespace)\s+(\w+)", text.split("{")[0])
    if m:
        return m.group(1)
    m = re.search(r"(\w+)\s*\([^;{]*\)\s*(?:const\s*)?\{", text)
    return m.group(1) if m else None


def auto_strip(lines):
    """Line numbers a contest library supplies: template, I/O driver, and named library code."""
    gone = set()
    for a, b in _top_level_items(lines):
        text = "\n".join(lines[a - 1 : b])
        head = text.lstrip()
        name = _item_name(text)
        if (
            head.startswith(("#", "using ", "typedef ", "import ", "package "))
            or re.search(r"\bdbgh?\b|operator\s*<<|operator\s*>>", text.split("{")[0])
            or _TEMPLATE_TYPE.match(head)
            or (name and (_LIBRARY_NAMES.match(name) or _LIBRARY_WORDS.search(name)))
        ):
            gone |= set(range(a, b + 1))
        elif name == "main":
            body = [k for k in range(a, b + 1) if lines[k - 1].strip() not in ("", "}")]
            calls_solve = re.search(r"\bsolve\s*\(", text) and len(body) <= 16
            for k in range(a, b + 1):
                if calls_solve or _IO_SETUP.search(lines[k - 1]):
                    gone.add(k)
    return gone


def strip_cpp_library(orig_path):
    """Return library-stripped C++ source using sibling orig.strip/.strip or auto_strip."""
    lines = _read_text(orig_path).splitlines()
    if os.path.basename(orig_path) == "orig.cpp":
        strip_file = os.path.join(os.path.dirname(orig_path), "orig.strip")
    else:
        strip_file = re.sub(r"_orig\.cpp$", ".strip", orig_path)
    gone = set()
    if os.path.isfile(strip_file):
        for line in _read_text(strip_file).splitlines():
            s = line.strip()
            if s and not s.startswith("#"):
                try:
                    a, b = map(int, s.split()[0].split("-"))
                    gone |= set(range(a, b + 1))
                except ValueError:
                    pass
    else:
        gone = auto_strip(lines)
    kept = [line for idx, line in enumerate(lines, 1) if idx not in gone]
    return "\n".join(kept)


def find_cpl_binary(override=None):
    """Locate the cpl binary (override > target/release/cpl > target/debug/cpl)."""
    if override:
        return override
    env_cpl = os.environ.get("CPL")
    if env_cpl and os.path.isfile(env_cpl):
        return env_cpl
    target_dir = os.environ.get("CARGO_TARGET_DIR") or os.path.join(ROOT, "target")
    for profile in ("release", "debug"):
        cand = os.path.join(target_dir, profile, "cpl")
        if os.path.isfile(cand):
            return cand
    return os.path.join(target_dir, "debug", "cpl")


def taste_counts(cpl, paths):
    """Per-file taste lint findings from `cpl lint --taste`: {path: (n, {code: n})}."""
    out = {}
    if not paths:
        return out
    r = sh([cpl, "lint", "--taste", *paths])
    for line in (r.stdout or "").splitlines():
        m = re.match(r"\s*(\d+)  (\S+)  (.*)$", line)
        if m and m.group(2) != "total":
            by = dict((c, int(n)) for c, n in (x.split(":") for x in m.group(3).split()))
            out[m.group(2)] = (int(m.group(1)), by)
    return out


_FIELD_RE = re.compile(r"^\s*(?:[-*]\s*)?\*\*([^*:]+):?\*\*:?\s*(.*)$")


_NON_FRICTION_SECTIONS = {
    "measurements",
    "measurement",
    "metrics",
    "summary",
    "algorithm",
    "algorithm summary",
    "what worked",
    "what worked well",
    "verification",
    "notes",
    "context",
    "status",
}


_AREA_ALIASES = {
    "graphs": "graphs",
    "graph": "graphs",
    "std.graph": "graphs",
    "trees": "trees",
    "tree": "trees",
    "std.tree": "trees",
    "ranges": "ranges",
    "range": "ranges",
    "strings": "strings",
    "string": "strings",
    "std.str": "strings",
    "math": "math",
    "number theory": "math",
    "std.math": "math",
    "std.mod": "math",
    "std.poly": "math",
    "std.matrix": "math",
    "flows": "flows",
    "flow": "flows",
    "std.flow": "flows",
    "containers": "containers",
    "container": "containers",
    "geometry": "geometry",
    "geom": "geometry",
    "std.geometry": "geometry",
    "dp": "dp",
    "slope": "dp",
    "std.slope": "dp",
    "seq": "seq",
    "std.seq": "seq",
    "core": "core",
    "core language": "core",
    "codegen": "core",
    "interp": "core",
    "syntax": "core",
    "check": "core",
    "types": "core",
    "tooling": "core",
    "cli": "core",
    "lsp": "core",
    "fmt": "core",
    "lint": "core",
    "stress": "core",
    "io": "core",
    "i/o": "core",
}


def _parse_explicit_area(raw):
    """Map an explicit `Area:` field value (taking its primary/leading area) to a canonical area, or None."""
    if not raw or not raw.strip():
        return None
    # Take the leading clause before '+', '/', '(', ',', or ';' and strip backticks/whitespace.
    primary = re.split(r"[+/(,;]", raw, maxsplit=1)[0].strip().strip("`'\"").strip().lower()
    if primary in _AREA_ALIASES:
        return _AREA_ALIASES[primary]
    # Try first token as a fallback (e.g. "strings 4.13" or "core syntax").
    first = primary.split()[0] if primary.split() else ""
    return _AREA_ALIASES.get(first)


def _kw_matches(kw, hay):
    if kw.isalpha() and len(kw) <= 4:
        return re.search(r"\b" + re.escape(kw) + r"\b", hay) is not None
    return kw in hay


def route_friction_area(title, body, fields):
    """Route a friction item to one of the 10 stdlib areas (or 'core').

    Prefers the explicit `Area:` field (including decorated forms like
    ``seq (`plan/STDLIB.md`)``, ``containers (`plan/STDLIB.md`) + codegen``,
    ``number theory (`plan/STDLIB.md`) + core language``, or ``core language``)
    and falls back to keyword matching only when `Area:` is missing or unrecognized.
    """
    explicit = _parse_explicit_area(fields.get("area", ""))
    if explicit is not None:
        return explicit
    hay = (title + "\n" + body).lower()
    for area, kws in _AREA_KEYWORDS:
        if any(_kw_matches(kw, hay) for kw in kws):
            return area
    return "core"


def parse_friction_file(path):
    """Parse one research/frictions/<NAME>.md file into structured friction items."""
    text = _read_text(path)
    rel = os.path.relpath(path, ROOT) if os.path.isabs(path) else path
    items = []
    warnings = []
    raw_sections = []
    for sec in re.split(r"(?m)^##\s+", text)[1:]:
        lines = sec.splitlines()
        if not lines:
            continue
        title = lines[0].strip()
        low = title.lower()
        if low in _NON_FRICTION_SECTIONS or low.startswith("status:"):
            continue
        if low in ("frictions", "friction", "friction items"):
            for sub in re.split(r"(?m)^###\s+", "\n".join(lines[1:]))[1:]:
                sub_lines = sub.splitlines()
                if sub_lines:
                    raw_sections.append((sub_lines[0].strip(), sub_lines[1:]))
            continue
        raw_sections.append((title, lines[1:]))

    for title, body_lines in raw_sections:
        fields = {}
        cur_key = None
        for line in body_lines:
            m = _FIELD_RE.match(line)
            if m:
                key = m.group(1).strip().rstrip(":").lower()
                val = m.group(2).strip()
                fields[key] = val
                cur_key = key
            elif cur_key and line.startswith("  ") and not line.strip().startswith("```"):
                pass

        # Severity
        raw_sev = fields.get("severity", "").strip().lower()
        sev_match = re.match(r"^(blocker|high|medium|low)\b", raw_sev)
        severity = sev_match.group(1) if sev_match else "unknown"
        if severity == "unknown":
            warnings.append(f"{rel} :: {title}: missing or unrecognized Severity ({raw_sev!r})")

        # Status (default open; closed/fixed/resolved or ~~title~~ marks closed)
        raw_status = fields.get("status", "").strip().lower()
        if title.startswith("~~") and title.endswith("~~"):
            status = "closed"
        elif any(w in raw_status for w in ("closed", "fixed", "resolved", "done")):
            status = "closed"
        else:
            status = "open"

        # Goals: normalize to canonical AGENTS.md goal names; warn on missing/unknown
        raw_goals = fields.get("goal it hurts", fields.get("goals", fields.get("goal", "")))
        raw_clean = re.sub(r"\([^)]*\)", "", raw_goals).lower()
        canonical_map = [
            ("express", ("express", "revise", "revisable", "guessable")),
            ("hard-to-get-wrong", ("hard-to-get-wrong", "hard to get wrong", "safety", "wrong")),
            ("performance", ("performance", "control", "fast", "speed")),
            ("doubt-to-evidence", ("doubt-to-evidence", "doubt to evidence", "evidence", "diagnos")),
            ("dependable-submission", ("dependable-submission", "dependable submission", "submission", "judge")),
            ("accumulate", ("accumulate", "library", "reuse")),
            ("coherence", ("coherence", "coherent")),
        ]
        goals = []
        for cname, needles in canonical_map:
            if any(n in raw_clean for n in needles):
                goals.append(cname)
        if not goals and raw_clean.strip():
            warnings.append(f"{rel} :: {title}: unrecognized Goal it hurts ({raw_goals!r})")
        elif not goals:
            warnings.append(f"{rel} :: {title}: missing Goal it hurts")

        area = route_friction_area(title, "\n".join(body_lines), fields)

        items.append(
            {
                "file": rel,
                "title": title.strip("~").strip(),
                "severity": severity,
                "goals": goals,
                "area": area,
                "status": status,
                "general_fix": fields.get("general fix", "")[:200],
            }
        )
    return items, warnings


def parse_review_score(port_name):
    path = os.path.join(rc.REVIEWS_DIR, f"{port_name}.md")
    if not os.path.isfile(path):
        return None
    score, _ = rc.parse_review_score_text(_read_text(path))
    return score


def measure_tokens(p):
    """Contestant-written token counts for a port against its original."""
    cp_toks = len(tokenize_cp(_read_text(p["abs_path"])))
    orig_toks = len(tokenize_cpp(_read_text(p["abs_orig"])))
    stripped_toks = len(tokenize_cpp(strip_cpp_library(p["abs_orig"])))
    dream_toks = dream_ratio = dream_gap = None
    if p.get("dream") and os.path.isfile(p["dream"]):
        dream_toks = len(tokenize_cp(_read_text(p["dream"])))
        dream_ratio = round(cp_toks / dream_toks, 4) if dream_toks else None
        dream_gap = cp_toks - dream_toks
    return {
        "cp_tokens": cp_toks,
        "orig_tokens": orig_toks,
        "token_ratio": round(cp_toks / orig_toks, 4) if orig_toks else None,
        "stripped_token_ratio": round(cp_toks / stripped_toks, 4) if stripped_toks else None,
        "dream_tokens": dream_toks,
        "dream_ratio": dream_ratio,
        "dream_gap_tokens": dream_gap,
        "taste_score": parse_review_score(p["name"]),
    }


def load_baseline(path):
    if not os.path.isfile(path):
        return None
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def token_summary(ports):
    """Median, max and over-1.0x count of the stripped token ratio, for port.cp and dream.cp."""
    def stats(ps):
        rs = sorted(r for r in (measure_tokens(p)["stripped_token_ratio"] for p in ps) if r)
        if not rs:
            return "none"
        return (f"median {rs[len(rs) // 2]:.2f}x, max {rs[-1]:.2f}x, "
                f"{sum(r > 1.0 for r in rs)} over 1.0x, n={len(rs)} "
                f"({len(ps) - len(rs)} skipped: the whole original strips as library)")
    dreams = [dict(p, abs_path=p["dream"], dream=None) for p in ports if p.get("dream")]
    return (f"port.cp/orig.cpp (library stripped): {stats(ports)}\n"
            f"dream.cp/orig.cpp (library stripped): {stats(dreams)}")


OUT = os.path.join(ROOT, "research", "PORTS.md")
SHOWN = 60


def load_scoreboard_perf():
    """Read cached runtime ratios from research/scoreboard.json (+ .ports.jsonl) without re-running."""
    try:
        data = load_baseline(DEFAULT_BASELINE)
    except Exception:
        return {}, None
    if not data:
        return {}, None
    by_id = {}
    for p in data.get("ports", []):
        perf = p.get("perf") or {}
        if perf.get("measured") and perf.get("time_ratio") is not None:
            by_id[p["id"]] = perf
    time_p50 = (data.get("totals") or {}).get("perf", {}).get("time_p50")
    return by_id, time_p50


def added_dates():
    log = subprocess.run(
        ["git", "log", "--diff-filter=A", "--format=@%aI", "--name-only", "--", "corpus/problems"],
        cwd=ROOT, capture_output=True, text=True, check=True).stdout
    dates, day = {}, None
    for line in log.splitlines():
        if line.startswith("@"):
            day = line[1:]
        elif line.endswith(".cp"):
            dates.setdefault(line, day)
    return dates


def statement_url(name):
    m = re.fullmatch(r"cf(\d+)([a-z]\d*)", name)
    if m:
        return f"https://codeforces.com/contest/{m[1]}/problem/{m[2].upper()}"
    m = re.fullmatch(r"(a[bgr]c)(\d+)([a-z])", name)
    if m:
        return f"https://atcoder.jp/contests/{m[1]}{m[2]}/tasks/{m[1]}{m[2]}_{m[3]}"
    return None


def friction(name):
    path = os.path.join(FRICTIONS_DIR, f"{name}.md")
    if not os.path.isfile(path):
        return None, None, 0, []
    text = _read_text(path)
    title = re.search(r"(?m)^# \S+\s*[—-]\s*(.+)$", text)
    items, _ = parse_friction_file(path)
    open_items = [it for it in items if it.get("status") == "open"]
    heads = [re.sub(r"^\d+\.\s*", "", it["title"]) for it in open_items]
    areas = []
    for it in open_items:
        a = it.get("area", "core")
        if a not in areas:
            areas.append(a)
    return (title[1].strip() if title else None), (heads[0] if heads else None), len(heads), areas


def cell(s, n=90):
    s = (s or "").replace("|", "\\|").replace("\n", " ")
    return s if len(s) <= n else s[: n - 1] + "…"


def worst_dream_leaderboard(rows, limit=20):
    """Return the worst `limit` ports by port/dream token ratio (deterministic tie-break)."""
    with_dream = [
        r for r in rows if r[6].get("dream_ratio") is not None and r[6].get("dream_tokens")
    ]
    with_dream.sort(
        key=lambda r: (
            -r[6]["dream_ratio"],
            -(r[6].get("dream_gap_tokens") or 0),
            r[1]["id"],
        )
    )
    return with_dream[:limit]


def main():
    if "--tokens" in sys.argv[1:]:
        print(token_summary(rc.discover_ports()))
        return
    dates = added_dates()
    perf_by_id, time_p50 = load_scoreboard_perf()
    ports = rc.discover_ports()
    taste_lints = taste_counts(find_cpl_binary(), [p["abs_path"] for p in ports])
    rows = []
    for p in ports:
        title, top, count, areas = friction(p["name"])
        tok = measure_tokens(p)
        rows.append((dates.get(p["path"], "uncommitted"), p, title, top, count, areas, tok, perf_by_id.get(p["id"])))
    rows.sort(key=lambda r: (r[0], r[1]["name"]), reverse=True)
    ratios = sorted(r[6]["stripped_token_ratio"] for r in rows if r[6]["stripped_token_ratio"])
    median = ratios[len(ratios) // 2] if ratios else None
    taste_scores = sorted(r[6]["taste_score"] for r in rows if r[6].get("taste_score") is not None)
    taste_med = taste_scores[len(taste_scores) // 2] if taste_scores else None
    rt_note = (
        f" Runtime is C++ build vs original on max-size input from `research/scoreboard.json`"
        f" (p50 {time_p50:.2f}x, goal at most {TIME_P50_GOAL:.1f}x)."
        if time_p50 is not None
        else " Runtime is C++ build vs original from `research/scoreboard.json`."
    )
    taste_note = (
        f" Taste is 1–5 from `research/reviews/` (median {taste_med}/5 across {len(taste_scores)} reviewed)."
        if taste_med is not None
        else ""
    )
    out = [
        "# Ports",
        "",
        "Generated by `scripts/ports-page.py`; do not edit by hand. The newest",
        f"{min(SHOWN, len(rows))} of {len(rows)} ports, each differential-tested against the accepted C++",
        "by `cpl` and the corpus gate. Tokens are contestant-written: the port",
        "against the original with its library code stripped"
        + (f" (median {median:.2f}x, goal at most 0.6x)." if median else ".")
        + rt_note
        + taste_note,
        "Dream is the same solution as we would like to write it, unconstrained",
        "by today's compiler, with its token count; the gap is what the language",
        "still owes.",
        "",
        "Lints counts the `cpl lint --taste` findings (the `handbook/process/porting.md`",
        f"checklist as lints L05xx; {sum(n for n, _ in taste_lints.values())} over all ports).",
        "",
        "| Added | Problem | Port | Tokens | Runtime | Taste | Lints | Areas | Frictions | First friction |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    for day, p, title, top, count, areas, tok, perf in rows[:SHOWN]:
        url = statement_url(p["name"])
        prob = f"[{p['name']}]({url})" if url else p["name"]
        if title:
            prob += f" {cell(title, 40)}"
        port = f"[.cp](../{p['path']}) · [C++](../{p['orig_path']})"
        if tok["dream_tokens"] is not None:
            dream_path = os.path.join(os.path.dirname(p["path"]), "dream", p["name"] + ".cp")
            port += f" · [dream](../{dream_path})"
        ratio = tok["stripped_token_ratio"] or tok["token_ratio"]
        toks = f"{tok['cp_tokens']} ({ratio:.2f}x)" if ratio else str(tok["cp_tokens"])
        if tok["dream_tokens"] is not None:
            toks += f", dream {tok['dream_tokens']}"
        rt = f"{perf['time_ratio']:.2f}x" if perf and perf.get("time_ratio") is not None else "–"
        if tok.get("taste_score") is not None:
            taste = f"[{tok['taste_score']}/5](reviews/{p['name']}.md)"
        else:
            taste = "–"
        area_cell = ", ".join(areas[:3]) if areas else "–"
        fr = f"[{count}](frictions/{p['name']}.md)" if count else "–"
        lints = taste_lints.get(p["abs_path"], (0, {}))[0]
        out.append(f"| {day[:10]} | {prob} | {port} | {toks} | {rt} | {taste} | {lints} | {area_cell} | {fr} | {cell(top)} |")

    leaderboard = worst_dream_leaderboard(rows, limit=20)
    if leaderboard:
        out.extend(
            [
                "",
                f"## Dream-gap leaderboard (worst {len(leaderboard)})",
                "",
                "Sorted worst-first by `cp / dream` token ratio so the polish lane works down the biggest gaps first.",
                "",
                "| # | Problem | Files | CP tokens | Dream tokens | Ratio | Gap | Areas | First friction |",
                "|---:|---|---|---:|---:|---:|---:|---|---|",
            ]
        )
        for rank, (_day, p, title, top, count, areas, tok, _perf) in enumerate(leaderboard, 1):
            url = statement_url(p["name"])
            prob = f"[{p['name']}]({url})" if url else p["name"]
            if title:
                prob += f" {cell(title, 36)}"
            dream_path = os.path.join(os.path.dirname(p["path"]), "dream", p["name"] + ".cp")
            files = f"[.cp](../{p['path']}) · [dream](../{dream_path})"
            gap = tok["cp_tokens"] - tok["dream_tokens"]
            gap_s = f"+{gap}" if gap >= 0 else str(gap)
            area_cell = ", ".join(areas[:3]) if areas else "–"
            fr_cell = f"[{cell(top, 70)}](frictions/{p['name']}.md)" if count and top else "–"
            out.append(
                f"| {rank} | {prob} | {files} | {tok['cp_tokens']} | {tok['dream_tokens']} | "
                f"{tok['dream_ratio']:.2f}x | {gap_s} | {area_cell} | {fr_cell} |"
            )

    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(out) + "\n")
    print(f"wrote {os.path.relpath(OUT, ROOT)}: {len(rows)} ports")


if __name__ == "__main__":
    main()
