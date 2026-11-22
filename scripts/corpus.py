#!/usr/bin/env python3
"""Organize the corpus: one corpus/problems/<problem>/meta.json per problem.

  scripts/corpus.py build   regenerate the generated fields of every meta.json
                            and corpus/index.json, keeping hand-written fields
  scripts/corpus.py check   fail when a meta.json is stale or names a missing file

Generated fields: problem, source, submissions, artifacts. Hand-written fields
(tags, difficulty, dream, spec, notes) survive a rebuild; the 100-problem
hill-climb fills them in.
"""
import json, os, re, sys
from collections import Counter, defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SUBS = os.path.join(ROOT, "corpus", "submissions")
PROBLEMS = os.path.join(ROOT, "corpus", "problems")
HAND = {"tags": [], "difficulty": None, "dream": "none", "spec": "none", "notes": ""}
STATUSES = {"none", "draft", "reviewed", "gold"}
EXT = {"CPP": ".cpp", "CPP03": ".cpp", "CPP11": ".cpp", "CPP14": ".cpp", "CPP17": ".cpp",
       "CPP20": ".cpp", "JAVA": ".java", "PY3": ".py", "PYPY3": ".py", "PY2": ".py", "C": ".c"}


def rel(p):
    return os.path.relpath(p, ROOT)


def canonical_id(problem):
    if re.fullmatch(r"dp[a-z]", problem):
        return f"atcoder_dp_{problem[2]}"
    return problem


def source_of(problem):
    if problem.startswith("atcoder_dp_"):
        return "atcoder_dp"
    m = re.match(r"[a-z]+", problem)
    return m.group(0) if m else problem


def artifacts():
    found = defaultdict(lambda: defaultdict(list))
    gold_dir = os.path.join(ROOT, "corpus", "gold")
    if os.path.isdir(gold_dir):
        for d, _, files in os.walk(gold_dir):
            for f in files:
                m = re.match(r"([a-z0-9_]+?)\.(cp)$", f)
                if m:
                    found[m.group(1)]["dream"].append(rel(os.path.join(d, f)))
    for q in os.listdir(PROBLEMS) if os.path.isdir(PROBLEMS) else []:
        pdir = os.path.join(PROBLEMS, q)
        if not os.path.isdir(pdir):
            continue
        for kind, fname in (("dream", "dream.cp"), ("spec", "spec.cp"), ("original", "orig.cpp")):
            f = os.path.join(pdir, fname)
            if os.path.exists(f):
                found[q][kind].insert(0, rel(f))
    return found


def generate():
    info = json.load(open(os.path.join(SUBS, "info.json")))
    by = defaultdict(list)
    for sid, e in info.items():
        by[canonical_id(e["problem"])].append((sid, e))
    arts = artifacts()
    all_problems = sorted(set(by.keys()) | set(arts.keys()))
    out = {}
    for problem in all_problems:
        subs = by.get(problem, [])
        rows = []
        for sid, e in sorted(subs, key=lambda x: x[1].get("date", "")):
            f = next((sid + x for x in (EXT.get(e.get("language", ""), ""), ".cpp", ".java", ".py")
                      if x and os.path.exists(os.path.join(SUBS, sid + x))), None)
            rows.append({"id": sid, "language": e.get("language"), "result": e.get("result"),
                         "time": e.get("time"), "memory_kb": e.get("memory"),
                         "file": rel(os.path.join(SUBS, f)) if f else None})
        p_arts = {k: sorted(v) for k, v in sorted(arts.get(problem, {}).items())}
        cpp_ac = sum(1 for r in rows if r["result"] == "AC" and (r["file"] or "").endswith(".cpp"))
        if not cpp_ac and p_arts.get("original"):
            cpp_ac = len(p_arts["original"])
        out[problem] = {
            "problem": problem,
            "source": source_of(problem),
            "submissions": rows,
            "cpp_ac": cpp_ac,
            "artifacts": p_arts,
        }
    return out


def merged(gen, path):
    old = json.load(open(path)) if os.path.exists(path) else {}
    meta = dict(gen)
    for k, v in HAND.items():
        meta[k] = old.get(k, v)
    return meta


def dump(x):
    text = json.dumps(x, indent=1)
    return re.sub(r"\{\n\s+(\"id\"[^{}]*?)\n\s+\}", lambda m: "{" + re.sub(r"\n\s+", " ", m.group(1)) + "}", text) + "\n"


def index(metas):
    return {"problems": len(metas),
            "with_cpp_ac": sum(1 for m in metas.values() if m["cpp_ac"]),
            "dream": {s: sum(1 for m in metas.values() if m["dream"] == s) for s in sorted(STATUSES)},
            "spec": {s: sum(1 for m in metas.values() if m["spec"] == s) for s in sorted(STATUSES)},
            "by_source": dict(sorted(Counter(m["source"] for m in metas.values()).items()))}


def main(mode):
    gen = generate()
    metas, errors = {}, []
    for problem, g in gen.items():
        path = os.path.join(PROBLEMS, problem, "meta.json")
        meta = merged(g, path)
        metas[problem] = meta
        for k in ("dream", "spec"):
            if meta[k] not in STATUSES:
                errors.append(f"{rel(path)}: {k} must be one of {sorted(STATUSES)}")
        for f in [r["file"] for r in meta["submissions"] if r["file"]] + \
                 [f for v in meta["artifacts"].values() for f in v]:
            if not os.path.exists(os.path.join(ROOT, f)):
                errors.append(f"{rel(path)}: missing {f}")
        if mode == "build":
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w").write(dump(meta))
        elif not os.path.exists(path) or open(path).read() != dump(meta):
            errors.append(f"{rel(path)}: stale; run scripts/corpus.py build")
    if os.path.isdir(PROBLEMS):
        for d in os.listdir(PROBLEMS):
            if os.path.isdir(os.path.join(PROBLEMS, d)) and d not in gen:
                errors.append(f"corpus/problems/{d}: no submissions or artifacts")
    ipath = os.path.join(ROOT, "corpus", "index.json")
    if mode == "build":
        open(ipath, "w").write(dump(index(metas)))
    elif not os.path.exists(ipath) or open(ipath).read() != dump(index(metas)):
        errors.append("corpus/index.json: stale; run scripts/corpus.py build")
    for e in errors:
        print(e, file=sys.stderr)
    print(f"corpus: {len(metas)} problems, {len(errors)} error(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in ("build", "check"):
        sys.exit(__doc__)
    sys.exit(main(sys.argv[1]))
