#!/usr/bin/env python3
"""Master corpus registry: one deduplicated scoreboard for every port in proto/ports/.

    python3 bench/corpus_registry.py                  # rewrite the registry and the report
    python3 bench/corpus_registry.py --run-harnesses  # first re-run every batch harness (about 15 minutes)
    python3 bench/corpus_registry.py --check          # exit 1 if the committed outputs are stale

It walks every `.cp` under proto/ports/, maps each file to a problem code in
corpus/submissions/info.json and to the accepted submission it was ported from,
groups the files by problem, picks one canonical port per problem, and measures
line counts two ways (raw and library-stripped, as proto/loc.py defines them).

Outputs, all under bench/:
    corpus-500-registry.json       every port file, every problem, every aggregate
    corpus-500-report.md           the same numbers as prose with showcase ports
    corpus-500-harness-runs.json   written by --run-harnesses: what each harness said

How a file is classified
    form        regression  proto/ports/*.cp and hillclimb_dp_graph/*.cp, run by proto/run_all.sh
                runnable    a batch file its harness builds: today/*.cp, or the top-level
                            file of a batch that has no today/ directory
                spec        a top-level file of a batch that also has today/: written against
                            the spec, no implementation accepts it
                draft       *_draft*.cp (blind-critic first drafts) and before/ (superseded)
                support     shared modules and bug repros that are not a port of a problem
    generation  v2.1 or v2 when the file header or the batch README names that spec, else v1
    verified    `pass` only when a recorded harness run printed a passing line for the port
                (or ran clean and names it); never inferred from a README

Line counts never include blank or comment-only lines. The stripped count of an original
uses the hand-written `<port>.strip` ranges when they exist; otherwise `auto_strip` below
removes the contest template, the I/O driver and library code recognised by name, and the
report says how far that heuristic is from the hand-written ranges.
"""
import argparse, glob, json, os, re, shutil, statistics, subprocess, sys, tempfile, time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS = os.path.join(ROOT, "proto", "ports")
SUBS = os.path.join(ROOT, "corpus", "submissions")
OUT_JSON = os.path.join(ROOT, "bench", "corpus-500-registry.json")
OUT_MD = os.path.join(ROOT, "bench", "corpus-500-report.md")
RUNS_JSON = os.path.join(ROOT, "bench", "corpus-500-harness-runs.json")
HARNESS_NAMES = ("check.py", "verify.py", "check.sh", "stress.sh")
EXT = {"CPP17": "cpp", "CPP20": "cpp", "CPP11": "cpp", "CLPP14": "cpp", "JAVA": "java"}

# contest family by problem-code prefix; anything else is "other"
FAMILIES = {
    "ioi": "IOI", "apio": "APIO", "ceoi": "CEOI", "btoi": "BtOI", "egoi": "EGOI", "joi": "JOI", "iati": "IATI",
    "noi": "NOI (China)", "noip": "NOIP", "noiwc": "NOI (China)", "noimps": "NOI (China)", "ctsc": "NOI (China)",
    "sdoi": "Chinese provincial", "shoi": "Chinese provincial", "jloi": "Chinese provincial", "csps": "NOIP",
    "cco": "CCO", "ccc": "CCC", "coci": "COCI", "coi": "COCI",
    "dmopc": "DMOPC", "dmpg": "DMOJ contests", "sac": "DMOJ contests", "yac": "DMOJ contests",
    "aac": "DMOJ contests", "wac": "DMOJ contests", "mccc": "DMOJ contests", "nccc": "DMOJ contests",
    "rccc": "DMOJ contests", "etsr": "DMOJ contests", "cheerio": "DMOJ contests", "tle": "DMOJ contests",
    "bsspc": "DMOJ contests", "acc": "DMOJ contests", "cmt": "DMOJ contests", "utso": "DMOJ contests",
    "wossoly": "DMOJ contests", "oly": "DMOJ contests", "tsoc": "DMOJ contests", "cpc": "DMOJ contests",
    "dwite": "DMOJ contests", "rpeng": "DMOJ contests", "ucrpc": "ICPC-style", "neerc": "ICPC-style",
    "dp": "AtCoder DP",
}
# families whose problems count as hard when they are still unported
HARD_FAMILIES = {"IOI", "APIO", "CEOI", "BtOI", "EGOI", "JOI", "IATI", "NOI (China)", "Chinese provincial", "CCO"}
HARD_STRIPPED_LOC = 60      # outside those families, an original this long (library-stripped) is hard too

LIBRARY_NAMES = re.compile(
    r"^(dsu|uf|unionfind|union_find|fenwick|bit|fenw|segtree|seg|segmenttree|lazyseg|lazy|sparsetable|sparse|rmq|lca|"
    r"mint|modint|mod_int|binpow|modpow|mpow|pw|power|modinv|inv|inverse|fact|choose|ncr|nck|binom|comb|sieve|"
    r"chash|custom_hash|hash|hasher|matrix|mat|cht|linecontainer|line|dinic|flow|maxflow|mcmf|hld|treap|trie|"
    r"kmp|zfunc|z_function|manacher|suffixarray|compress|dijkstra|scc|tarjan|fdiv|cdiv|floor_div|ceil_div|"
    r"ckmin|ckmax|chmin|chmax|gcd|lcm|extgcd|exgcd|rng|rnd|rand_int|rev|reversed|bset|ost|min_heap|pq)$", re.I)
LIBRARY_WORDS = re.compile(
    r"seg_?tree|segment_?tree|fenwick|sparse_?table|union_?find|disjoint_?set|mod_?int|montgomery|"
    r"suffix_?(array|automaton)|aho_?corasick|li_?chao|link_?cut|centroid|heavy_?light|convex_?hull|"
    r"min_?cost|max_?flow|bipartite_?match|hopcroft|y_?combinator|fast_?(io|input|output|scanner|reader)|"
    r"^(input)?reader$|^kattio$|^fastio$", re.I)
TEMPLATE_TYPE = re.compile(r"^template\s*<[^;{]*>\s*(struct|class)\b", re.S)   # a generic type is library code
IO_SETUP = re.compile(r"sync_with_stdio|cin\.tie|cin\.exceptions|freopen|setprecision")


def rel(path):
    return os.path.relpath(path, ROOT)


def read_lines(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read().split("\n")


def counted(lines, block_comments):
    """1-based numbers of the lines that count, exactly as proto/loc.py counts them"""
    out, inside = [], False
    for i, line in enumerate(lines, 1):
        s = line.strip()
        if inside:
            inside = "*/" not in s
            continue
        if not s or s.startswith("//"):
            continue
        if block_comments and s.startswith("/*"):
            inside = "*/" not in s
            continue
        out.append(i)
    return out


# ---------------------------------------------------------------- library stripping

def hand_strip(path):
    gone = set()
    for line in open(path):
        if line.strip() and not line.startswith("#"):
            a, b = map(int, line.split()[0].split("-"))
            gone |= set(range(a, b + 1))
    return gone


def top_level_items(lines):
    """(first, last) 1-based line spans of the top-level items of a C++/Java file"""
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
            if re.match(r"#\s*if", s): ifdef += 1
            if re.match(r"#\s*endif", s): ifdef -= 1
        else:
            code = re.sub(r'"(\\.|[^"\\])*"|\'(\\.|[^\'\\])*\'|//.*', "", s)
            depth += code.count("{") - code.count("}")
        done = (pre and not cont and ifdef <= 0) or (not pre and ifdef <= 0 and depth <= 0 and s.endswith((";", "}")))
        if done:
            items.append((start, i))
            start, depth = None, 0
    if start is not None:
        items.append((start, len(lines)))
    return items


def item_name(text):
    m = re.search(r"\b(?:struct|class|namespace)\s+(\w+)", text.split("{")[0])
    if m:
        return m.group(1)
    m = re.search(r"(\w+)\s*\([^;{]*\)\s*(?:const\s*)?\{", text)
    return m.group(1) if m else None


def auto_strip(lines):
    """line numbers a perfect contest library would have supplied: the template (includes,
    macros, aliases, debug printers), the I/O driver, and library code recognised by name"""
    gone = set()
    for a, b in top_level_items(lines):
        text = "\n".join(lines[a - 1:b])
        head = text.lstrip()
        name = item_name(text)
        if (head.startswith(("#", "using ", "typedef ", "import ", "package "))
                or re.search(r"\bdbgh?\b|operator\s*<<|operator\s*>>", text.split("{")[0])
                or TEMPLATE_TYPE.match(head)
                or (name and (LIBRARY_NAMES.match(name) or LIBRARY_WORDS.search(name)))):
            gone |= set(range(a, b + 1))
        elif name == "main":
            body = [k for k in range(a, b + 1) if lines[k - 1].strip() not in ("", "}")]
            calls_solve = re.search(r"\bsolve\s*\(", text) and len(body) <= 16
            for k in range(a, b + 1):
                if calls_solve or IO_SETUP.search(lines[k - 1]):
                    gone.add(k)
    return gone


def measure_original(path, strip_path=None):
    lines = read_lines(path)
    live = counted(lines, True)
    if strip_path and os.path.exists(strip_path):
        gone, how = hand_strip(strip_path), "hand"
    else:
        gone, how = auto_strip(lines), "auto"
    return len(live), len([i for i in live if i not in gone]), how


# ---------------------------------------------------------------- scanning

def load_corpus():
    info = json.load(open(os.path.join(SUBS, "info.json")))
    by_problem = defaultdict(list)
    for sid, row in info.items():
        by_problem[row["problem"]].append(sid)
    return info, by_problem


def submission_path(info, sid):
    ext = EXT.get(info[sid]["language"])
    path = os.path.join(SUBS, f"{sid}.{ext}") if ext else None
    return path if path and os.path.exists(path) else None


def best_accepted(info, by_problem, problem):
    """the latest accepted C++ submission of a problem, else the latest accepted Java one"""
    for want in ("cpp", "java"):
        ids = [s for s in by_problem.get(problem, []) if info[s]["result"] == "AC"
               and EXT.get(info[s]["language"]) == want and submission_path(info, s)]
        if ids:
            return max(ids, key=lambda s: info[s]["date"])
    return None


def family(problem):
    m = re.match(r"[a-z]+", problem)
    prefix = m.group(0) if m else ""
    if re.fullmatch(r"dp[a-z]", problem):
        prefix = "dp"
    return FAMILIES.get(prefix, "other")


def declared_generation(text):
    if re.search(r"(?<![\w/])v2\.1\b", text):
        return "v2.1"
    if re.search(r"CP Lang v2\b|locked v2\b|v2[- ]spec\b|spec v2\b|v2 sheet|v2 (ports|forms|solution)", text, re.I):
        return "v2"
    return None


def scan_ports(info, by_problem):
    problems = set(by_problem)
    readme_gen, has_today, harness = {}, {}, {}
    for d in sorted(glob.glob(PORTS + "/*/")):
        name = os.path.basename(d.rstrip("/"))
        readme = os.path.join(d, "README.md")
        readme_gen[name] = declared_generation(open(readme, errors="replace").read()) if os.path.exists(readme) else None
        has_today[name] = os.path.isdir(os.path.join(d, "today"))
        harness[name] = next((rel(os.path.join(d, h)) for h in HARNESS_NAMES if os.path.exists(os.path.join(d, h))), None)
    ports = []
    for path in sorted(glob.glob(PORTS + "/**/*.cp", recursive=True)):
        parts = os.path.relpath(path, PORTS).split(os.sep)
        group = parts[0] if len(parts) > 1 else "top-level"
        sub = parts[1] if len(parts) > 2 else None
        stem = parts[-1][:-3]
        base = re.sub(r"_draft\d*$", "", stem)
        lines = read_lines(path)
        header = [l for l in lines[:25] if l.startswith("//")]
        ids = [m for l in header for m in re.findall(r"\b(\d{7})\b", l) if m in info]
        sid, how = None, None
        if ids:
            sid, problem, how = ids[0], info[ids[0]]["problem"], "header"
        elif base in info:
            sid, problem, how = base, info[base]["problem"], "file name is the submission id"
        elif base in problems:
            problem, how = base, "file name is the problem code"
        else:
            problem = None
        if group in ("top-level", "hillclimb_dp_graph") and sub is None:
            form = "regression"
        elif sub == "before" or stem != base:
            form = "draft"
        elif problem is None or sub == "bugs":
            form = "support"
        elif sub == "today" or not has_today.get(group):
            form = "runnable"
        else:
            form = "spec"
        if form == "regression":
            cover = "proto/run_all.sh"
        elif form == "runnable":
            cover = harness.get(group)
        else:
            cover = None
        original, strip = None, None
        if problem:
            sibling = path[:-3] + "_orig.cpp"
            if sub == "before":
                sibling = os.path.join(PORTS, group, stem + "_orig.cpp")
            if os.path.exists(sibling):
                original, strip = sibling, sibling[:-9] + ".strip"
            else:
                if sid is None:      # look for "<id>" next to the problem code in the batch's harness and README
                    near = os.path.join(PORTS, group) if group != "top-level" else PORTS
                    for f in [os.path.join(near, h) for h in HARNESS_NAMES + ("README.md", "loc.py")]:
                        if os.path.exists(f):
                            for line in open(f, errors="replace"):
                                if re.search(rf"\b{re.escape(base)}\b", line):
                                    hit = [m for m in re.findall(r"\b(\d{7})\b", line) if m in info and info[m]["problem"] == problem]
                                    if hit:
                                        sid, how = hit[0], how + f", id from {os.path.basename(f)}"
                                        break
                        if sid:
                            break
                if sid is None:
                    sid = best_accepted(info, by_problem, problem)
                    how += ", latest accepted submission" if sid else ""
                original = submission_path(info, sid) if sid else None
        row = {
            "path": rel(path), "group": group, "form": form, "problem": problem, "mapped_by": how,
            "submission": sid, "original": rel(original) if original else None,
            "generation": declared_generation("\n".join(header)) or readme_gen.get(group) or "v1",
            "cp_loc": len(counted(lines, False)),
            "wish_markers": sum("WISH" in l for l in lines),
            "harness": cover,
        }
        if original:
            row["cpp_raw_loc"], row["cpp_stripped_loc"], row["strip_source"] = measure_original(original, strip)
            row["original_language"] = "java" if original.endswith(".java") else "cpp"
        ports.append(row)
    return ports


# ---------------------------------------------------------------- harness runs

def harness_list():
    hs = [rel(os.path.join(d, h)) for d in sorted(glob.glob(PORTS + "/*/")) for h in HARNESS_NAMES
          if os.path.exists(os.path.join(d, h))]
    return hs + ["proto/run_all.sh"]


PASS = re.compile(r"\bOK\b|\bok\b|\bPASS(ED)?\b|\bpass(ed)?\b|agree|✓|\bverified\b|\bvalidated\b", re.I)
FAIL = re.compile(r"FAIL|differ|mismatch|\bWA\b|error|Traceback|not checked|skipp?ed|SKIP|timeout|crash", re.I)


def port_verdicts(harness, log, stems):
    """what the harness printed about each port: the last line that names it decides"""
    out = {}
    for stem in stems:
        hits = [l for l in log if re.search(rf"(?<![\w/]){re.escape(stem)}(?![\w])", l)]
        if not hits:
            continue
        last = hits[-1]
        out[stem] = {"verdict": "fail" if FAIL.search(last) else "pass" if PASS.search(last) else "unclear",
                     "line": last.strip()[:200]}
        why = next((l for l in log if f"/{stem}.cp:" in l), None)      # the prototype's own diagnostic, if any
        if why and out[stem]["verdict"] == "fail":
            out[stem]["diagnostic"] = re.sub(r"^.*?\.cp:\d+:\s*", "", why.strip()).replace("`", "'")[:200]
    return out


def run_harnesses(jobs, iters, timeout):
    """run every harness in a throwaway copy of the repository (they stamp files) and record the result"""
    tmp = tempfile.mkdtemp(prefix="corpus-registry-")
    copy = os.path.join(tmp, "repo")
    shutil.copytree(ROOT, copy, ignore=shutil.ignore_patterns(".git", "target", "node_modules", "dist"))
    # the last commit that touched what the harnesses read, so the stamp survives a rebase of bench/ changes
    commit = subprocess.run(["git", "log", "-1", "--format=%h", "--", "proto", "corpus"], cwd=ROOT,
                            capture_output=True, text=True).stdout.strip()

    def one(h):
        cmd = ["python3", h] if h.endswith(".py") else ["bash", h]
        start = time.time()
        try:
            p = subprocess.run(cmd, cwd=copy, capture_output=True, text=True, errors="replace", timeout=timeout,
                               env=dict(os.environ, ITERS=str(iters)))
            code, text = p.returncode, p.stdout + p.stderr
        except subprocess.TimeoutExpired as e:
            code, text = None, (e.stdout or b"").decode(errors="replace") if isinstance(e.stdout, bytes) else (e.stdout or "")
        print(f"{h}: exit {code} in {time.time() - start:.0f}s", file=sys.stderr)
        return h, {"exit_code": code, "timed_out": code is None, "seconds": round(time.time() - start),
                   "log": text.split("\n")}

    with ThreadPoolExecutor(jobs) as ex:
        runs = dict(ex.map(one, harness_list()))
    shutil.rmtree(tmp, ignore_errors=True)
    return {"commit": commit, "iters_for_run_all": iters, "runs": runs}


def summarise_runs(raw, ports):
    """keep the exit code and one verdict line per port; drop the full logs"""
    stems = defaultdict(set)
    for p in ports:
        if p["harness"]:
            stems[p["harness"]].add(os.path.basename(p["path"])[:-3])
    out = {"commit": raw["commit"], "iters_for_run_all": raw["iters_for_run_all"], "runs": {}}
    for h, run in sorted(raw["runs"].items()):
        out["runs"][h] = {"exit_code": run["exit_code"], "timed_out": run["timed_out"], "seconds": run["seconds"],
                          "ports": port_verdicts(h, run["log"], sorted(stems[h])),
                          "tail": [l[:200] for l in run["log"] if l.strip()][-3:]}
    return out


def apply_verification(ports, runs):
    for p in ports:
        h = p["harness"]
        if not h:
            p["verified"] = "no harness"
            continue
        run = (runs or {}).get("runs", {}).get(h)
        if not run:
            p["verified"] = "not run"
            continue
        v = run["ports"].get(os.path.basename(p["path"])[:-3])
        if v:
            p["verified"] = v["verdict"]
            p["harness_line"] = v["line"]
            if "diagnostic" in v:
                p["diagnostic"] = v["diagnostic"]
        else:
            p["verified"] = "not reported"      # the harness ran and never named this port


# ---------------------------------------------------------------- dedup and aggregates

FORM_RANK = {"regression": 0, "runnable": 1, "spec": 2, "draft": 3}
GEN_RANK = {"v2.1": 0, "v2": 1, "v1": 2}


def canonical_key(p):
    return (p["verified"] != "pass", FORM_RANK[p["form"]], GEN_RANK[p["generation"]], p["cp_loc"], p["path"])


def build_problems(ports):
    groups = defaultdict(list)
    for p in ports:
        if p["problem"] and p["form"] != "support":
            groups[p["problem"]].append(p)
    problems = {}
    for code, rows in sorted(groups.items()):
        real = [p for p in rows if p["form"] != "draft"] or rows
        best = min(real, key=canonical_key)
        specs = [p for p in rows if p["form"] == "spec"]
        spec = min(specs, key=lambda p: (GEN_RANK[p["generation"]], p["cp_loc"], p["path"])) if specs else None
        problems[code] = {
            "family": family(code),
            "canonical": best["path"],
            "canonical_form": best["form"],
            "canonical_verified": best["verified"],
            "spec_port": spec["path"] if spec else None,
            "duplicates": [p["path"] for p in sorted(rows, key=canonical_key) if p is not best],
            "port_files": len(rows),
            "groups": sorted({p["group"] for p in rows}),
        }
    return problems


def loc_stats(rows):
    """aggregate line counts over ports that have a measured original"""
    rows = [p for p in rows if p.get("cpp_raw_loc")]
    if not rows:
        return {"ports": 0}
    cp, raw, strip = (sum(p[k] for p in rows) for k in ("cp_loc", "cpp_raw_loc", "cpp_stripped_loc"))
    red = lambda ours, theirs: round(100.0 * (theirs - ours) / theirs, 1) if theirs else 0.0
    per_raw = [red(p["cp_loc"], p["cpp_raw_loc"]) for p in rows]
    per_strip = [red(p["cp_loc"], p["cpp_stripped_loc"]) for p in rows]
    return {
        "ports": len(rows),
        "cp_loc": {"total": cp, "mean": round(cp / len(rows), 1), "median": statistics.median(p["cp_loc"] for p in rows)},
        "cpp_raw_loc": {"total": raw, "mean": round(raw / len(rows), 1), "median": statistics.median(p["cpp_raw_loc"] for p in rows)},
        "cpp_stripped_loc": {"total": strip, "mean": round(strip / len(rows), 1),
                             "median": statistics.median(p["cpp_stripped_loc"] for p in rows)},
        "raw_reduction_pct": {"of_totals": red(cp, raw), "mean": round(statistics.mean(per_raw), 1),
                              "median": round(statistics.median(per_raw), 1)},
        "stripped_reduction_pct": {"of_totals": red(cp, strip), "mean": round(statistics.mean(per_strip), 1),
                                   "median": round(statistics.median(per_strip), 1)},
        "no_stripped_win": sum(p["cp_loc"] >= p["cpp_stripped_loc"] for p in rows),
    }


def breakdown(rows, key):
    groups = defaultdict(list)
    for p in rows:
        groups[key(p)].append(p)
    return {k: loc_stats(v) for k, v in sorted(groups.items())}


def strip_calibration(ports):
    """how the auto-strip heuristic compares with the hand-written .strip ranges, on the ports that have them"""
    hand = [p for p in ports if p.get("strip_source") == "hand" and p["form"] == "regression"]
    auto_total, hand_total, diffs = 0, 0, []
    for p in hand:
        lines = read_lines(os.path.join(ROOT, p["original"]))
        live = counted(lines, True)
        gone = auto_strip(lines)
        a = len([i for i in live if i not in gone])
        auto_total += a
        hand_total += p["cpp_stripped_loc"]
        diffs.append(a - p["cpp_stripped_loc"])
    return {"ports": len(hand), "hand_stripped_total": hand_total, "auto_stripped_total": auto_total,
            "auto_over_hand_pct": round(100.0 * (auto_total - hand_total) / hand_total, 1) if hand_total else 0.0,
            "exact_matches": sum(d == 0 for d in diffs), "within_5_lines": sum(abs(d) <= 5 for d in diffs)}


def unported_inventory(info, by_problem, problems):
    rows = []
    for code in sorted(by_problem):
        if code in problems:
            continue
        accepted = [s for s in by_problem[code] if info[s]["result"] == "AC"]
        sid = best_accepted(info, by_problem, code)
        row = {"problem": code, "family": family(code), "accepted_submissions": len(accepted),
               "attempts": len(by_problem[code]), "submission": sid}
        if sid:
            path = submission_path(info, sid)
            row["original"] = rel(path)
            row["cpp_raw_loc"], row["cpp_stripped_loc"], _ = measure_original(path)
            row["hard"] = row["family"] in HARD_FAMILIES or row["cpp_stripped_loc"] >= HARD_STRIPPED_LOC
        else:
            row["hard"] = False
            row["note"] = "no accepted C++ or Java source" if accepted else "never accepted"
        rows.append(row)
    return rows


def build_registry(runs):
    info, by_problem = load_corpus()
    ports = scan_ports(info, by_problem)
    apply_verification(ports, runs)
    problems = build_problems(ports)
    by_path = {p["path"]: p for p in ports}
    canon = [by_path[v["canonical"]] for v in problems.values()]
    spec = [by_path[v["spec_port"]] for v in problems.values() if v["spec_port"]]
    verified = [p for p in canon if p["verified"] == "pass"]
    unported = unported_inventory(info, by_problem, problems)
    count = lambda rows, key: {k: sum(1 for p in rows if key(p) == k) for k in sorted({key(p) for p in rows})}
    best_form = lambda v: by_path[v["canonical"]]
    summary = {
        "cp_files": len(ports),
        "cp_files_by_form": count(ports, lambda p: p["form"]),
        "cp_files_by_generation": count([p for p in ports if p["form"] != "support"], lambda p: p["generation"]),
        "unique_problems": len(problems),
        "problems_with_more_than_one_file": sum(v["port_files"] > 1 for v in problems.values()),
        "problems_ported_in_more_than_one_group": sum(len(v["groups"]) > 1 for v in problems.values()),
        "corpus_problems": len(by_problem),
        "corpus_problems_accepted": sum(any(info[s]["result"] == "AC" for s in ids) for ids in by_problem.values()),
        "verification": {
            "problems_verified": len(verified),
            "problems_by_canonical_status": count(canon, lambda p: p["verified"]),
            "problems_spec_only": sum(best_form(v)["form"] == "spec" for v in problems.values()),
            "files_by_status": count([p for p in ports if p["form"] in ("regression", "runnable")], lambda p: p["verified"]),
            "harness_commit": (runs or {}).get("commit"),
            "harnesses_run": len((runs or {}).get("runs", {})),
            "harnesses_clean_exit": sum(r["exit_code"] == 0 for r in (runs or {}).get("runs", {}).values()),
        },
        "loc": {
            "canonical": loc_stats(canon),
            "canonical_verified": loc_stats(verified),
            "spec_form": loc_stats(spec),
            "spec_form_same_problems_runnable": loc_stats([by_path[v["canonical"]] for v in problems.values()
                                                           if v["spec_port"] and best_form(v)["form"] != "spec"]),
            "canonical_by_family": breakdown(canon, lambda p: family(p["problem"])),
            "canonical_by_generation": breakdown(canon, lambda p: p["generation"]),
            "canonical_by_form": breakdown(canon, lambda p: p["form"]),
            "spec_form_by_generation": breakdown(spec, lambda p: p["generation"]),
            "canonical_by_strip_source": breakdown(canon, lambda p: p.get("strip_source", "none")),
        },
        "strip_calibration": strip_calibration(ports),
        "unported": {
            "problems": len(unported),
            "with_accepted_source": sum(1 for u in unported if u["submission"]),
            "hard": sum(u["hard"] for u in unported),
            "hard_by_family": count([u for u in unported if u["hard"]], lambda u: u["family"]),
        },
    }
    return {"summary": summary, "problems": problems, "ports": ports, "unported": unported}


# ---------------------------------------------------------------- report

def snippet(path, limit=40):
    lines = read_lines(os.path.join(ROOT, path))
    while lines and (lines[0].startswith("// cp:") or not lines[0].strip()):
        lines = lines[1:]
    lines = [l.rstrip() for l in lines]
    while lines and not lines[-1]:
        lines.pop()
    if len(lines) > limit:
        lines = lines[:limit] + ["// ... (" + str(len(lines) - limit) + " more lines)"]
    return "```\n" + "\n".join(lines) + "\n```"


def showcases(reg):
    """deterministic picks: verified canonical ports of substantial originals, largest stripped
    reduction first, one per contest family"""
    by_path = {p["path"]: p for p in reg["ports"]}
    picks, seen = [], set()
    rows = [by_path[v["canonical"]] for v in reg["problems"].values()]
    rows = [p for p in rows if p["verified"] == "pass" and p.get("cpp_stripped_loc", 0) >= 40 and 6 <= p["cp_loc"] <= 32
            and p["wish_markers"] == 0]
    rows.sort(key=lambda p: (p["cp_loc"] / p["cpp_stripped_loc"], p["path"]))
    for p in rows:
        fam = family(p["problem"])
        if fam not in seen:
            seen.add(fam)
            picks.append(p)
        if len(picks) == 4:
            break
    return picks


def spec_showcase(reg):
    by_path = {p["path"]: p for p in reg["ports"]}
    rows = []
    for v in reg["problems"].values():
        c = by_path[v["canonical"]]
        if v["spec_port"] and c["form"] == "runnable" and c["verified"] == "pass":
            s = by_path[v["spec_port"]]
            if 8 <= s["cp_loc"] <= 24 and c["cp_loc"] >= s["cp_loc"] + 8:
                rows.append((s["cp_loc"] / c["cp_loc"], s["path"], s, c))
    return min(rows)[2:] if rows else None


def write_report(reg):
    s, L = reg["summary"], reg["summary"]["loc"]
    v, c, cv, sp = s["verification"], L["canonical"], L["canonical_verified"], L["spec_form"]
    forms, gens = s["cp_files_by_form"], s["cp_files_by_generation"]
    by_path = {p["path"]: p for p in reg["ports"]}
    out = []
    w = out.append
    w("# Corpus registry: the deduplicated port scoreboard\n")
    w("Generated by `python3 bench/corpus_registry.py`; do not edit by hand. The full data, one row per port "
      "file and per problem, is in [`corpus-500-registry.json`](corpus-500-registry.json).\n")
    w("## Headline\n")
    w(f"- **{s['cp_files']} `.cp` files** under `proto/ports/` are ports of **{s['unique_problems']} unique problems**, "
      f"out of {s['corpus_problems_accepted']} problems with an accepted submission in the corpus "
      f"({s['corpus_problems']} attempted).")
    w(f"- **{v['problems_verified']} of those {s['unique_problems']} problems** have a port that a harness "
      f"differentially checked against the original in a recorded run"
      + (f" at `{v['harness_commit']}`" if v["harness_commit"] else "") + ".")
    w(f"- Canonical ports are **{c['raw_reduction_pct']['of_totals']}% shorter than the raw originals** and "
      f"**{c['stripped_reduction_pct']['of_totals']}% shorter than the library-stripped originals** "
      f"({c['cp_loc']['total']} CP Lang lines against {c['cpp_raw_loc']['total']} raw and {c['cpp_stripped_loc']['total']} stripped).")
    w(f"- **{s['unported']['hard']} hard problems** with an accepted source are still unported.\n")

    w("## Files against problems\n")
    w(f"The {s['cp_files']} files are not {s['cp_files']} solved problems. {forms.get('regression', 0)} are regression ports "
      f"that `proto/run_all.sh` stresses on every change. {forms.get('runnable', 0)} are runnable scout ports, the form a batch "
      f"harness builds. {forms.get('spec', 0)} are spec-form files: the same solutions written against the locked spec, "
      f"which no implementation accepts yet, so each has a runnable twin in `today/`. {forms.get('draft', 0)} are blind-critic "
      f"first drafts or superseded `before/` versions, and {forms.get('support', 0)} are shared modules and a bug repro rather "
      f"than ports.\n")
    w(f"After grouping by problem code, {s['problems_with_more_than_one_file']} problems have more than one file and "
      f"{s['problems_ported_in_more_than_one_group']} were ported independently in more than one directory (parallel scout "
      f"batches landing the same pick, or a scout sketch that later graduated to a regression port). The registry keeps one "
      f"canonical port per problem: a port with a passing harness run beats one without, then regression beats runnable "
      f"beats spec, then the newer spec generation, then the shorter file. Every other file is listed under that problem's "
      f"`duplicates`.\n")
    multi = sorted(((len(p["groups"]), code, p) for code, p in reg["problems"].items() if len(p["groups"]) > 2), reverse=True)[:5]
    if multi:
        w("The most re-ported problems: " + "; ".join(
            f"`{code}` ({', '.join(p['groups'])})" for _, code, p in multi) + ".\n")

    w("## Line counts\n")
    w("Blank and comment-only lines never count. \"Stripped\" removes from the original what a perfect C++ contest "
      "library would have supplied: the template, the I/O driver and standard data structures.\n")
    w(f"Over the {c['ports']} canonical ports that have a measured original, the median original is "
      f"{c['cpp_raw_loc']['median']:g} lines raw and {c['cpp_stripped_loc']['median']:g} stripped, and the median port is "
      f"{c['cp_loc']['median']:g} lines (means: {c['cpp_raw_loc']['mean']}, {c['cpp_stripped_loc']['mean']} and "
      f"{c['cp_loc']['mean']}). Per port, the median reduction is {c['raw_reduction_pct']['median']}% raw and "
      f"{c['stripped_reduction_pct']['median']}% stripped (means {c['raw_reduction_pct']['mean']}% and "
      f"{c['stripped_reduction_pct']['mean']}%). {c['no_stripped_win']} ports are no shorter than their stripped original.\n")
    if cv.get("ports"):
        w(f"Restricted to the {cv['ports']} canonical ports with a passing harness run, the numbers are "
          f"{cv['raw_reduction_pct']['of_totals']}% raw and {cv['stripped_reduction_pct']['of_totals']}% stripped "
          f"({cv['cp_loc']['total']} lines against {cv['cpp_raw_loc']['total']} and {cv['cpp_stripped_loc']['total']}). "
          f"This is the number to quote: every line in it compiled and agreed with the original.\n")
    if sp.get("ports"):
        twin = L["spec_form_same_problems_runnable"]
        w(f"The {sp['ports']} spec-form files are shorter still: {sp['raw_reduction_pct']['of_totals']}% raw and "
          f"{sp['stripped_reduction_pct']['of_totals']}% stripped ({sp['cp_loc']['total']} lines). "
          + (f"The runnable ports of {twin['ports']} of those problems take {twin['cp_loc']['total']} lines "
             f"({twin['stripped_reduction_pct']['of_totals']}% stripped), so the gap between the spec and what builds today "
             f"is the difference between those two figures. " if twin.get("ports") else "")
          + "Spec-form numbers are a design target, not a measurement of working code.\n")
    cal = s["strip_calibration"]
    hand = L["canonical_by_strip_source"].get("hand", {}).get("ports", 0)
    w(f"How the stripping was done: {hand} canonical ports have hand-written `.strip` ranges, and those are used as they are. "
      f"For the rest, `auto_strip` removes the template, the driver, generic (`template`) types and top-level code "
      f"whose name is a library name (`DSU`, `binpow`, `SparseTable` and so on). On the {cal['ports']} regression ports that have hand-written ranges, "
      f"the heuristic leaves {cal['auto_stripped_total']} lines where the hand ranges leave {cal['hand_stripped_total']} "
      f"({cal['auto_over_hand_pct']:+}%), matching exactly on {cal['exact_matches']} and within five lines on "
      f"{cal['within_5_lines']}. It misses library code with a problem-specific name, so auto-stripped reductions "
      f"lean high.\n")

    w("### By spec generation\n")
    gen = L["canonical_by_generation"]
    for g, label in (("v1", "v1 (before the v2 lock)"), ("v2", "v2"), ("v2.1", "v2.1")):
        if g in gen and gen[g].get("ports"):
            x = gen[g]
            w(f"- **{label}**: {x['ports']} canonical ports, {x['raw_reduction_pct']['of_totals']}% raw and "
              f"{x['stripped_reduction_pct']['of_totals']}% stripped (median port {x['cp_loc']['median']:g} lines).")
    sg = L["spec_form_by_generation"]
    parts = [f"{g}: {sg[g]['ports']} files, {sg[g]['raw_reduction_pct']['of_totals']}% raw and "
             f"{sg[g]['stripped_reduction_pct']['of_totals']}% stripped" for g in ("v1", "v2", "v2.1") if sg.get(g, {}).get("ports")]
    if parts:
        w("- Spec-form files by generation: " + "; ".join(parts) + ".")
    w("")
    w(f"A file's generation is what its header or its batch README declares; {gens.get('v1', 0)} files are v1, "
      f"{gens.get('v2', 0)} are v2 and {gens.get('v2.1', 0)} are v2.1. "
      + ("No port declares v2.1 yet: the v2.1 lock landed after the latest batches were written. " if not gens.get("v2.1") else "")
      + "The canonical port of a v2 batch is usually its runnable `today/` form, which is restricted to what the "
        "prototype builds, so the v2 canonical figures understate the v2 spec; the spec-form line above is the one "
        "that shows it.\n")

    w("### By contest family\n")
    fam = L["canonical_by_family"]
    for name, x in sorted(fam.items(), key=lambda kv: -kv[1].get("ports", 0)):
        if x.get("ports"):
            w(f"- **{name}**: {x['ports']} problems, {x['cpp_raw_loc']['total']} raw / {x['cpp_stripped_loc']['total']} stripped "
              f"/ {x['cp_loc']['total']} CP Lang lines, {x['raw_reduction_pct']['of_totals']}% and "
              f"{x['stripped_reduction_pct']['of_totals']}% shorter.")
    w("")

    w("## Verification coverage\n")
    st = v["problems_by_canonical_status"]
    fs = v["files_by_status"]
    if v["harnesses_run"]:
        w(f"All {v['harnesses_run']} harnesses (`proto/run_all.sh` and each batch's `check.py`, `verify.py`, `check.sh` or "
          f"`stress.sh`) were run from a clean copy of the repository at `{v['harness_commit']}`; "
          f"{v['harnesses_clean_exit']} exited 0. A port counts as verified only when the run printed a passing line "
          f"that names it. The per-harness exit codes and the line behind every verdict are in "
          f"[`corpus-500-harness-runs.json`](corpus-500-harness-runs.json).\n")
    else:
        w("No harness run is recorded, so nothing below is verified. Run `python3 bench/corpus_registry.py --run-harnesses`.\n")
    w(f"By problem, on the canonical port: {st.get('pass', 0)} pass, {st.get('fail', 0)} fail, "
      f"{st.get('unclear', 0) + st.get('not reported', 0)} are covered by a harness that did not report a clear verdict for them, "
      f"{st.get('not run', 0)} have a harness that was not run, and {st.get('no harness', 0)} have no harness at all "
      f"({v['problems_spec_only']} of those exist only as a spec-form sketch). By file, of the "
      f"{sum(fs.values())} regression and runnable files: " + ", ".join(f"{n} {k}" for k, n in sorted(fs.items())) + ".\n")
    bad = defaultdict(list)
    for code, p in sorted(reg["problems"].items()):
        port = by_path[p["canonical"]]
        if port["verified"] in ("fail", "unclear", "not reported"):
            bad[port["harness"]].append((code, port))
    if bad:
        rows = [port for ports in bad.values() for _, port in ports]
        rejected = sum("diagnostic" in port for port in rows)
        w(f"The {len(rows)} problems whose best port did not get a pass are below. {rejected} of them never ran: the "
          f"prototype rejects the port at build time, most often because a library name the port was written against "
          f"is not in the prototype's library at this commit. The rest ran and disagreed with the original, or the "
          f"harness skips them.\n")
        for h, ports in sorted(bad.items()):
            w(f"- `{h}`: " + ", ".join(f"`{code}`" + (f" (`{port['diagnostic']}`)" if "diagnostic" in port
                                                      else f" ({port['verified']})") for code, port in ports))
        w("")
    none = defaultdict(list)
    for code, p in reg["problems"].items():
        if by_path[p["canonical"]]["verified"] == "no harness":
            none[by_path[p["canonical"]]["group"]].append(code)
    if none:
        w("Problems with no harness, by directory: " + "; ".join(f"`{g}` ({len(cs)})" for g, cs in sorted(none.items())) + ".\n")

    w("## Showcase ports\n")
    w("Picked by the tool, not by hand: verified canonical ports of originals with at least 40 stripped lines, "
      "largest stripped reduction first, one per contest family.\n")
    for p in showcases(reg):
        w(f"**`{p['problem']}`** ([`{p['path']}`](../{p['path']})): {p['cp_loc']} lines against "
          f"{p['cpp_raw_loc']} raw and {p['cpp_stripped_loc']} stripped in [`{os.path.basename(p['original'])}`](../{p['original']}).\n")
        w(snippet(p["path"]) + "\n")
    pair = spec_showcase(reg)
    if pair:
        sfile, run = pair
        w(f"And one spec-against-today pair, the widest gap among short ports. `{sfile['problem']}` as the spec lets it be "
          f"written ([`{sfile['path']}`](../{sfile['path']}), {sfile['cp_loc']} lines, not buildable yet):\n")
        w(snippet(sfile["path"]) + "\n")
        w(f"The same solution as it builds and passes today ([`{run['path']}`](../{run['path']}), {run['cp_loc']} lines):\n")
        w(snippet(run["path"]) + "\n")

    w("## Still unported\n")
    u = s["unported"]
    hard = sorted((x for x in reg["unported"] if x["hard"]), key=lambda x: (-x["cpp_stripped_loc"], x["problem"]))
    w(f"{u['problems']} corpus problems have no port; {u['with_accepted_source']} of them have an accepted C++ or Java "
      f"source to port from. {u['hard']} of those count as hard: an olympiad-family problem ("
      + ", ".join(sorted(HARD_FAMILIES)) + f") or any original of {HARD_STRIPPED_LOC} or more stripped lines. By family: "
      + ", ".join(f"{k} {n}" for k, n in sorted(u["hard_by_family"].items(), key=lambda kv: -kv[1])) + ".\n")
    if hard:
        w("The longest twenty, by stripped lines of the latest accepted submission:\n")
        for x in hard[:20]:
            w(f"- `{x['problem']}` ({x['family']}): {x['cpp_stripped_loc']} stripped lines, `{x['original']}`")
        w("")
        w("The full list is the `unported` array of the registry JSON.\n")
    with open(OUT_MD, "w") as f:
        f.write("\n".join(out).rstrip() + "\n")


def dump(obj):
    return json.dumps(obj, indent=1, sort_keys=False, ensure_ascii=False) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--run-harnesses", action="store_true", help="re-run every harness and rewrite corpus-500-harness-runs.json")
    ap.add_argument("--jobs", type=int, default=4, help="harnesses to run at once (default 4)")
    ap.add_argument("--iters", type=int, default=60, help="ITERS for proto/run_all.sh (default 60)")
    ap.add_argument("--timeout", type=int, default=3000, help="seconds allowed per harness (default 3000)")
    ap.add_argument("--check", action="store_true", help="fail if the committed registry or report is out of date")
    args = ap.parse_args()
    runs = json.load(open(RUNS_JSON)) if os.path.exists(RUNS_JSON) else None
    if args.run_harnesses:
        info, by_problem = load_corpus()
        runs = summarise_runs(run_harnesses(args.jobs, args.iters, args.timeout), scan_ports(info, by_problem))
        open(RUNS_JSON, "w").write(dump(runs))
    reg = build_registry(runs)
    if args.check:
        old_json, old_md = open(OUT_JSON).read(), open(OUT_MD).read()
        write_report(reg)
        stale = dump(reg) != old_json or open(OUT_MD).read() != old_md
        open(OUT_MD, "w").write(old_md)
        print("stale: re-run bench/corpus_registry.py" if stale else "up to date")
        sys.exit(1 if stale else 0)
    open(OUT_JSON, "w").write(dump(reg))
    write_report(reg)
    s = reg["summary"]
    print(f"{s['cp_files']} .cp files, {s['unique_problems']} unique problems, "
          f"{s['verification']['problems_verified']} verified; "
          f"raw -{s['loc']['canonical']['raw_reduction_pct']['of_totals']}%, "
          f"stripped -{s['loc']['canonical']['stripped_reduction_pct']['of_totals']}%; "
          f"{s['unported']['hard']} hard problems unported")


if __name__ == "__main__":
    main()
