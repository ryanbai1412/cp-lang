#!/usr/bin/env python3
"""Std views, keys and ops compose: every pair, lazy against materialized, on both engines.

Enforces std-api/R14 (handbook/guidelines/std-api-checklist.md) for handbook/domains/stdlib.md's
"Composes" row. The pairs come from std itself, through `cpl doc --format json std`:

  view      a pub std fn with a sequence-trait parameter that returns a view (`yields`, or a std
            type with a `Seq` impl): `rev`, `items`, `filter`, `windows`, `zip`, ...
  consumer  a pub std fn with a sequence-trait parameter (every view is one too)
  key       a pub std fn `f<T>(x: T)` returning a std type with an `Ord` impl (`desc`), passed as `by:`
  op        an operation value the checker knows (`is_op_text` in crates/cpl-check), passed as `op:`

A source is a view applied to a base list (Ints, a Str, optionals, lists). Each consumer
takes each source in each of its sequence slots whose trait the source has, with each key for
its `by:` and each op for its `op:`. The brute force is the same call on the source
materialized: `List(src)`, keeping keys (`.keyed(keys:)` for `Keyed`, `.rekey(src.keys)` for
`Indexed`). A pair whose brute force checks is a case: its lazy form must check too, and print
the same result as the brute force under `cpl run` and `cpl run --release`. A consumer with no
case fails too, so a new std fn can't go unexercised.

  scripts/std-compose.py [--cpl PATH] [--list] [--only REGEX]
"""
import argparse
import json
import os
import re
import resource
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRAITS = ["Seq", "Bidir", "Keyed", "Positional", "Indexed", "Sized"]
BASES = {  # tried in order; a case whose brute force outgrows LIMITS moves on to the next that checks
    "xs": "[3, 1, 4, 1, 5, 2]",
    "cs": '"abcab"',
    "os": "[Some(3), None, Some(1), Some(4)]",
    "ls": "[[3, 1], [], [4]]",
    "x2": "[3, 1]",
    "c2": '"ab"',
    "o2": "[Some(3), None]",
    "l2": "[[3, 1], []]",
}
LIMITS = (4 << 30, 300)  # bytes of memory and seconds for one run of a case program
PRED = 'x => "{x}".len % 2 == 1'
KEY = 'x => "{x}".len'
ARGS = {  # argument for a required parameter, by the shape of its type
    r"Int|Len|-?\d*\.\.[\w^+\-]*": "2",
    r"Range<\w+>": "0..2",
    r"\w+ -> Bool": PRED,
    r"\(\w+, \w+\) -> Bool": '(a, b) => "{a}" <= "{b}"',
    r"\w+ -> \?\w+": 'x => if "{x}".len % 2 == 1: Some("{x}".len) else: None',
    r"\(\w+, \w+\) -> \w+": '(a, b) => if "{a}" <= "{b}": a else: b',
    r"\w+ -> \w+": KEY,
    r"Str": '","',
}


def split_top(s, sep=","):
    out, depth, cur = [], 0, ""
    for i, c in enumerate(s):
        if c in "<([{":
            depth += 1
        elif c in ")]}" or (c == ">" and s[i - 1] != "-"):
            depth -= 1
        if c == sep and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += c
    return [x for x in out + [cur.strip()] if x]


def parse(sig):
    m = re.match(r"pub fn (\w+)", sig)
    rest, gen = sig[m.end():], ""
    if rest.startswith("<"):
        depth = 0
        for i, c in enumerate(rest):
            depth += {"<": 1, ">": -1 if rest[i - 1] != "-" else 0}.get(c, 0)
            if depth == 0:
                gen, rest = rest[1:i], rest[i + 1:]
                break
    depth = 0
    for i, c in enumerate(rest):
        depth += {"(": 1, ")": -1}.get(c, 0)
        if depth == 0:
            params, rest = rest[1:i], rest[i + 1:]
            break
    ret = re.sub(r"\s+where .*", "", rest.split("->", 1)[1].strip()) if "->" in rest else "()"
    gens = {n.strip(): b.strip() for n, _, b in (g.partition(":") for g in split_top(gen))}
    ps = []
    for p in split_top(params.replace(";", ",")):
        name, _, ty = p.partition(":")
        ty, _, default = ty.partition("=")
        ps.append({"name": name.replace("mut ", "").replace("..", "").strip(), "variadic": ".." in name,
                   "mut": "mut " in name, "ty": ty.strip(), "default": default.strip() or None})
    return {"name": m.group(1), "gens": gens, "params": ps, "ret": ret}


def slot_trait(f, p):
    m = re.match(r"(\w+)", f["gens"].get(p["ty"], ""))
    return m.group(1) if m and m.group(1) in TRAITS and not p["mut"] else None


def checker_ops(root):
    text = (root / "crates/cpl-check/src/check/removed.rs").read_text(encoding="utf-8")
    body = re.search(r"fn is_op_text.*?const OPS: \[&str; \d+\] = \[(.*?)\]", text, re.S).group(1)
    return re.findall(r'"([^"]+)"', body)


def std_index(cpl, root):
    doc = json.loads(subprocess.run([cpl, "doc", "--format", "json", str(root / "std")], capture_output=True,
                                    text=True, check=True, env={**os.environ, "CPL_STD_DIR": str(root / "std")}).stdout)
    texts = {p.name: p.read_text(encoding="utf-8") for p in (root / "std").glob("*.cp")}
    impls = lambda trait: {m for t in texts.values() for m in re.findall(
        rf"^impl(?:<[^\n]*?>)? {trait}(?:<[^\n]*?>)? for (\w+)", t, re.M)}
    seq_types, ord_types = impls("Seq"), impls("Ord")
    fns = []
    for mod in doc["modules"]:
        lines = texts.get(Path(mod["path"]).name, "").splitlines()
        for x in mod["items"]:
            if x["kind"] != "fn" or not x["signature"].startswith("pub fn"):
                continue
            f = parse(x["signature"])
            src = lines[x["line"] - 1] if x["line"] <= len(lines) else ""
            f["get"], f["where"] = src.lstrip().startswith("@get"), f"{mod['path']}:{x['line']}"
            fns.append(f)
    consumers = [f for f in fns if any(slot_trait(f, p) for p in f["params"])
                 and not any(p["variadic"] for p in f["params"])]
    is_view = lambda ret: ret.startswith("yields ") or (re.match(r"\w+", ret) or re.match("", "")).group(0) in seq_types
    views = [f for f in consumers if is_view(f["ret"])]
    keys = [f["name"] for f in fns if len(f["params"]) == 1 and f["params"][0]["ty"] in f["gens"]
            and (re.match(r"\w+", f["ret"]) or re.match("", "")).group(0) in ord_types]
    return consumers, views, keys, checker_ops(root), is_view


def call(f, args):
    """`f` applied to `args` ({param: expr}), method syntax on its `self`."""
    ps = f["params"]
    pos = [args[p["name"]] for p in ps if p["name"] in args and not p["default"]]
    named = [f"{p['name']}: {args[p['name']]}" for p in ps if p["name"] in args and p["default"]]
    if ps and ps[0]["name"] == "self":
        recv, rest = pos[0], pos[1:] + named
        return f"({recv}).{f['name']}" + ("" if f["get"] and not rest else f"({', '.join(rest)})")
    return f"{f['name']}({', '.join(pos + named)})"


def fill(f, slots, elem, extra):
    """Arguments for `f`: `slots` for its sequence params, `extra` named options, synthesized the rest."""
    args = dict(slots)
    for p in f["params"]:
        if p["name"] in args or p["default"] or p["name"] in extra:
            continue
        if p["ty"] in f["gens"] and not slot_trait(f, p):
            args[p["name"]] = elem
            continue
        arg = next((a for pat, a in ARGS.items() if re.fullmatch(pat, p["ty"])), None)
        if arg is None:
            return None
        args[p["name"]] = arg
    return {**args, **extra}


def normal(f, expr, is_view):
    ret = f["ret"]
    if ret.startswith("(domain"):
        return f"List(({expr}).1)"
    if is_view(ret):
        inner = ret[len("yields "):] if ret.startswith("yields ") else ""
        return f"List(({expr}).map(q => List(q)))" if re.search(r"\bS\b", inner) else f"List({expr})"
    return expr


def materialized(src, trait):
    if trait == "Keyed":
        return f"List({src}).keyed(keys: List(({src}).items.map(q => q.0)))"
    if trait == "Indexed":
        return f"List({src}).rekey(({src}).keys)"
    return f"List({src})"


def cases(consumers, views, keys, ops, is_view):
    """Candidate cases by unit, each unit a pair to cover: view into consumer slot, key or op into
    consumer, view with key or op. A unit's candidates go base by base; its first that checks is its case."""
    def source(v, b):
        if v is None:
            return b
        slots = {q["name"]: b for q in v["params"] if slot_trait(v, q)}
        args = fill(v, slots, f"List({b})[0]", {})
        return call(v, args) if args else None

    def variants(c):
        out = [{}]
        if any(p["name"] == "by" for p in c["params"]):
            out += [{"by": k} for k in keys]
        if any(p["name"] == "op" for p in c["params"]):
            out = [{**v, "op": o} for v in out for o in ops]
        return out

    def case(b, c, p, v, extra):
        src = source(v, b)
        if src is None:
            return None
        trait = slot_trait(c, p)
        others = {q["name"]: b for q in c["params"] if slot_trait(c, q) and q is not p}
        lazy = fill(c, {**others, p["name"]: src}, f"List({src})[0]", extra)
        if lazy is None:
            return None
        lazy_src = v is not None and is_view(v["ret"])  # else `src` is a value with nothing to materialize
        brute = {**lazy, p["name"]: materialized(src, trait)} if lazy_src else lazy
        return {"base": b, "consumer": c["name"], "where": c["where"], "trait": trait, "src": src,
                "lazy": normal(c, call(c, lazy), is_view), "brute": normal(c, call(c, brute), is_view)}

    units = {}
    for c in consumers:
        vs, slots = variants(c), [p for p in c["params"] if slot_trait(c, p)]
        for p in slots:
            for v in [None, *views]:
                units[("view", c["name"], p["name"], v and v["name"])] = [
                    (b, c, p, v, vs[0]) for b in BASES]
        for x in vs[1:]:
            units[("variant", c["name"], str(x))] = [(b, c, slots[0], None, x) for b in BASES]
    for v in views:
        for x in [{"by": k} for k in keys] + [{"op": o} for o in ops]:
            units[("with", v["name"], str(x))] = [
                (b, c, p, v, {**variants(c)[0], **x}) for b in BASES for c in consumers
                if all(any(q["name"] == k for q in c["params"]) for k in x)
                for p in c["params"] if slot_trait(c, p)]
    return {u: [y for y in (case(*x) for x in xs) if y] for u, xs in units.items()}


def header():
    uses = ", ".join(TRAITS)
    mods = sorted(f.stem for f in (ROOT / "std").glob("*.cp"))
    lines = [f"use std.{m}.*" for m in mods] + [f"use std.seq.{{{uses}}}", "use std.algebra.{desc}"]
    lines += [f"let {n} = {e}" for n, e in BASES.items()]
    lines += [f"fn need_{t.lower()}<S: {t}>(s: S) = ()" for t in TRAITS]
    return lines


def check(cpl, fns, chunk=200):
    """{fn name: ok} for `fns` ({name: body}), `cpl check` over files of `chunk` fns in parallel."""
    names = list(fns)

    def one(part):
        lines, at = header(), {}
        for name in part:
            at[len(lines) + 1] = name
            lines.append(f"fn {name}() = {fns[name]}")
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "probe.cp"
            path.write_text("\n".join(lines) + "\n", encoding="utf-8")
            r = subprocess.run([cpl, "check", "--json", str(path)], capture_output=True, text=True,
                               env={**os.environ, "CPL_STD_DIR": str(ROOT / "std")})
        try:
            files = json.loads(r.stdout)["files"]
        except (json.JSONDecodeError, KeyError):
            keep = Path(tempfile.mkdtemp(prefix="std-compose-")) / "probe.cp"
            keep.write_text("\n".join(lines) + "\n", encoding="utf-8")
            sys.exit(f"std-compose: `cpl check` exits {r.returncode} on {keep}:\n{r.stderr[-3000:]}")
        bad = {at.get(d["start"]["line"]) for f in files for d in f["diagnostics"] if d["severity"] == "error"}
        if None in bad:
            sys.exit(f"std-compose: the probe header fails to check:\n{r.stdout[:2000]}")
        return {name: name not in bad for name in part}

    # The checker reports a failing generic instantiation once per file, at one of its sites, so any
    # verdict from a file with errors is checked again in smaller files, down to one probe per file.
    print(f"std-compose: checking {len(names)} probes", file=sys.stderr, flush=True)
    ok, todo = {}, names
    size = max(1, min(chunk, -(-len(todo) // (os.cpu_count() or 1))))
    with ThreadPoolExecutor(os.cpu_count()) as pool:
        while todo:
            again = []
            for part in pool.map(one, [todo[i:i + size] for i in range(0, len(todo), size)]):
                final = all(part.values()) or len(part) == 1
                for k, v in part.items():
                    (ok.__setitem__(k, v) if final else again.append(k))
            todo, size = again, max(1, size // 8)
    return ok


def program(ids, path):
    """`ids` as one program at `path`, read `start mode`: mode 0 runs every case from position `start`
    on, comparing lazy to brute; mode 1 and 2 run only the case at `start`, its lazy or its brute form
    against itself. The file stays put so `--release` builds it once."""
    lines, at = header() + ["let start = read<Int>()", "let mode = read<Int>()"], {}
    for i, c in ids.items():
        at.update({len(lines) + 1: ("l", i), len(lines) + 2: ("a", i), len(lines) + 3: ("b", i)})
        lines += [f"fn l{i}() -> Bool = {c['lazy']} == {c['brute']}", f"fn a{i}() -> Bool = {c['lazy']} == {c['lazy']}",
                  f"fn b{i}() -> Bool = {c['brute']} == {c['brute']}"]
    for j, i in enumerate(ids):
        lines += [f'if mode == 0 and start <= {j}: print(if l{i}(): "{j} ok" else: "{j} differs")',
                  f'if mode == 1 and start == {j}: print(if a{i}(): "{j} ok" else: "{j} differs")',
                  f'if mode == 2 and start == {j}: print(if b{i}(): "{j} ok" else: "{j} differs")']
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return at


def generic_flags(cpl):
    """The compile line of judge `generic`, as `cpl judges` says it."""
    lines = subprocess.run([cpl, "judges"], capture_output=True, text=True, check=True).stdout.splitlines()
    return lines[next(i for i, ln in enumerate(lines) if ln.split()[:2] == ["*", "generic"]) + 1].split()


def engines(cpl, path):
    """{engine: argv} that run `path`: the interpreter, and the C++ `cpl run --release` builds, built here
    once with judge `generic`'s compile line and the release build's checks however long the C++ is."""
    cpp, exe = path.with_suffix(".cpp"), path.with_suffix(".bin")
    env = {**os.environ, "CPL_STD_DIR": str(ROOT / "std")}
    subprocess.run([cpl, "build", str(path), "--full", "--checks", "traps", "-o", str(cpp)],
                   capture_output=True, check=True, env=env)
    r = subprocess.run([*generic_flags(cpl), "-o", str(exe), str(cpp)], capture_output=True, text=True)
    if r.returncode:
        keep = Path(tempfile.mkdtemp(prefix="std-compose-"))
        for f in (path, cpp):
            (keep / f.name).write_text(f.read_text(encoding="utf-8"), encoding="utf-8")
        return {"interp": [cpl, "run", str(path)]}, f"{keep / cpp.name}: the C++ fails to compile:\n{r.stderr[:3000]}"
    return {"interp": [cpl, "run", str(path)], "native": [str(exe)]}, None


def run(argv, start, mode):
    """{position: printed verdict} for one run of `argv`, and the run."""
    limit = lambda: resource.setrlimit(resource.RLIMIT_AS, (LIMITS[0], LIMITS[0]))
    try:
        r = subprocess.run(argv, input=f"{start} {mode}\n", capture_output=True, text=True, timeout=LIMITS[1],
                           env={**os.environ, "CPL_STD_DIR": str(ROOT / "std")}, preexec_fn=limit)
    except subprocess.TimeoutExpired as e:
        out = e.stdout.decode(errors="replace") if isinstance(e.stdout, bytes) else e.stdout or ""
        r = subprocess.CompletedProcess(argv, None, out, "timed out")
    seen = dict(ln.split(" ", 1) for ln in r.stdout.splitlines() if ln.split(" ")[0].isdigit())
    return {int(k): v for k, v in seen.items()}, r


def outgrew(r):
    """Whether run `r` stopped at LIMITS: out of time, or out of memory on either engine."""
    return r.returncode is None or any(m in r.stderr for m in ("memory allocation of", "bad_alloc"))


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--cpl", default=str(ROOT / "target/release/cpl"))
    ap.add_argument("--list", action="store_true", help="print the cases and stop")
    ap.add_argument("--only", help="keep the cases whose consumer or source matches this regex")
    a = ap.parse_args()
    consumers, views, keys, ops, is_view = std_index(a.cpl, ROOT)
    units = cases(consumers, views, keys, ops, is_view)
    if a.only:
        units = {u: xs for u, xs in units.items() if any(re.search(a.only, str(p)) for p in u)}
    srcs = sorted({x["src"] for xs in units.values() for x in xs})
    traits = check(a.cpl, {f"t{i}_{t}": f"need_{t.lower()}({s})" for i, s in enumerate(srcs) for t in TRAITS})
    has = {s: {t for t in TRAITS if traits[f"t{i}_{t}"]} for i, s in enumerate(srcs)}
    units = {u: [x for x in xs if x["trait"] in has[x["src"]]] for u, xs in units.items()}
    tried, good = set(), set()

    def pick(want):
        """{unit: its first case that checks} over `want` ({unit: candidates}), probing base by base."""
        found = {}
        for b in BASES:
            todo = {u: [x for x in xs if x["base"] == b] for u, xs in want.items() if u not in found}
            probe = sorted({x["brute"] for xs in todo.values() for x in xs} - tried)
            ok = check(a.cpl, {f"e{i}": f"{e} == {e}" for i, e in enumerate(probe)})
            good.update(e for i, e in enumerate(probe) if ok[f"e{i}"])
            tried.update(probe)
            for u, xs in todo.items():
                hit = next((x for x in xs if x["brute"] in good), None)
                if hit:
                    found[u] = hit
        return found

    found = pick(units)
    print(f"std-compose: {len(views)} views, {len(consumers)} consumers, {len(keys)} keys, {len(ops)} ops; "
          f"{len(units)} pairs, {len(found)} cases")
    if a.list:
        for c in {x["lazy"]: x for x in found.values()}.values():
            print(f"{c['lazy']}  vs  {c['brute']}")
        return 0
    failed = [f"{c['where']}: `{c['name']}` has no case: no source checks in its sequence slots"
              for c in consumers if not a.only and c["name"] not in {x["consumer"] for x in found.values()}]

    def chunk_run(todo):
        """(findings, cases run, lazy forms whose brute force outgrew LIMITS) for the cases `todo`."""
        out, heavy, retry, n = [], set(), {}, 0
        whole = len(todo)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "pairs.cp"
            while todo:
                at = program(todo, path)
                r = subprocess.run([a.cpl, "check", "--json", str(path)], capture_output=True, text=True,
                                   env={**os.environ, "CPL_STD_DIR": str(ROOT / "std")})
                bad = {at.get(x["start"]["line"]) for f in json.loads(r.stdout)["files"]
                       for x in f["diagnostics"] if x["severity"] == "error"}
                if not bad:
                    break
                if None in bad:
                    return [(None, f"the pairs program fails to check outside its cases: {r.stdout[-2000:]}")], 0, heavy
                # One error per failing instantiation lands on some site of it; a blamed case is judged alone.
                blamed = {i for k, i in bad}
                alone = check(a.cpl, {f"{k}{i}": f"{todo[i][x]} == {todo[i][y]}" for i in blamed
                                      for k, x, y in (("l", "lazy", "brute"), ("a", "lazy", "lazy"), ("b", "brute", "brute"))})
                for i in sorted(blamed):
                    if not alone[f"b{i}"]:
                        out.append((todo[i]["lazy"], f"{todo[i]['where']}: the brute force stops checking: {todo[i]['brute']}"))
                    elif not (alone[f"l{i}"] and alone[f"a{i}"]):
                        out.append((todo[i]["lazy"], f"{todo[i]['where']}: the lazy form fails to check where its"
                                    f" brute force checks: {todo[i]['lazy']}"))
                    else:
                        retry[i] = todo[i]
                todo = {i: c for i, c in todo.items() if i not in blamed}
            cs = list(todo.values())
            n = len(cs)
            argvs, broken = engines(a.cpl, path) if cs else ({}, None)
            out += [(None, broken)] if broken else []
            for name, argv in argvs.items():
                start = 0
                while start < len(cs):
                    seen, r = run(argv, start, 0)
                    out += [(cs[j]["lazy"], f"{name}: {cs[j]['lazy']} differs from {cs[j]['brute']}")
                            for j, v in seen.items() if v != "ok"]
                    stop = next((j for j in range(start, len(cs)) if j not in seen), None)
                    if stop is None:
                        break
                    brute, rb = run(argv, stop, 2)
                    if not brute and outgrew(rb):
                        heavy.add(cs[stop]["lazy"])
                        start = stop + 1
                        continue
                    lazy, _ = run(argv, stop, 1)
                    if bool(lazy) != bool(brute):
                        out.append((cs[stop]["lazy"], f"{name}: {cs[stop]['lazy']} {'fails to run' if brute else 'runs'}"
                                    f" where {cs[stop]['brute']} {'runs' if brute else 'fails to run'}:"
                                    f" {r.stderr.strip()[-1500:]}"))
                    start = stop + 1
        if whole == 1 and retry:
            return out + [(c["lazy"], f"{c['where']}: checks alone but not in the pairs program: {c['lazy']}")
                          for c in retry.values()], n, heavy
        parts = [retry] if len(retry) < whole else [dict(list(retry.items())[:whole // 2]), dict(list(retry.items())[whole // 2:])]
        for part in parts:
            if part:
                o, m, h = chunk_run(part)
                out, n, heavy = out + o, n + m, heavy | h
        return out, n, heavy

    total, want = 0, units
    while found:
        cases_ = list({x["lazy"]: x for x in found.values()}.values())
        print(f"std-compose: running {len(cases_)} cases on both engines", file=sys.stderr, flush=True)
        heavy = set()
        with ThreadPoolExecutor(os.cpu_count()) as pool:
            for out, n, h in pool.map(chunk_run, [dict(enumerate(cases_[k:k + 100], k))
                                                  for k in range(0, len(cases_), 100)]):
                heavy |= h
                failed += [msg for lazy, msg in out if lazy not in h]
                total += n - len(h)
        order = list(BASES)
        want = {u: [x for x in want[u] if order.index(x["base"]) > order.index(c["base"])]
                for u, c in found.items() if c["lazy"] in heavy}
        failed += [f"{found[u]['where']}: {found[u]['lazy']} outgrows {LIMITS[0] >> 30} GiB or {LIMITS[1]} s"
                   " on every base whose brute force checks" for u, xs in want.items() if not xs]
        found = pick({u: xs for u, xs in want.items() if xs})
    print(f"std-compose: {total} cases ran on both engines")
    print("\n".join(failed) if failed else "std-compose: every pair agrees on both engines")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
