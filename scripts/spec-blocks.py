#!/usr/bin/env python3
"""Check every code block in spec/core and spec/lib against today's compiler.

Prints, per chapter, how many blocks check with the spec stubs as the
standard library, then the most common first errors. The count is a floor
for the 0.2 -> 0.3 delta (spec/delta/0.2.md): many blocks are fragments.

`--save` records the blocks that check in spec/blocks-ok.txt (a hash per
block); `--ratchet` fails when a recorded block stops checking, so a spec
edit can't silently break an example that used to be valid.
"""
import collections, glob, hashlib, os, re, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CPL = os.path.join(ROOT, "target", "release", "cpl")
env = {**os.environ, "CPL_SPEC": "1", "CPL_STD": os.path.join(ROOT, "spec", "std")}
BASE = os.path.join(ROOT, "spec", "blocks-ok.txt")
mode = next((a for a in sys.argv[1:] if a.startswith("--")), "")
saved = set(open(BASE).read().split()) if os.path.exists(BASE) else set()
passing, broke = set(), []
per, errs, first = collections.defaultdict(lambda: [0, 0]), collections.Counter(), {}
with tempfile.TemporaryDirectory() as tmp:
    for f in sorted(glob.glob(os.path.join(ROOT, "spec/core/*.md")) + glob.glob(os.path.join(ROOT, "spec/lib/*.md"))):
        name = os.path.relpath(f, ROOT)
        for i, block in enumerate(re.findall(r"```(?:cp|python)\n(.*?)```", open(f).read(), re.S)):
            p = os.path.join(tmp, f"b{i}.cp")
            open(p, "w").write(block)
            r = subprocess.run([CPL, "check", p], capture_output=True, text=True, env=env)
            m = re.search(r"error\[(E\d+)\]: ([^\n]*)", r.stdout + r.stderr)
            h = hashlib.sha1(block.encode()).hexdigest()[:16]
            if not m:
                passing.add(h)
            elif h in saved:
                broke.append(f"{name} block {i}: {m.group(1)} {m.group(2)[:70]}")
            per[name][1] += 1
            if m:
                errs[m.group(1)] += 1
                first.setdefault(m.group(1), f"{name} block {i}: {m.group(2)[:70]}")
            else:
                per[name][0] += 1
ok, total = sum(v[0] for v in per.values()), sum(v[1] for v in per.values())
print(f"spec blocks: {ok}/{total} check")
for k, (a, b) in sorted(per.items()):
    print(f"  {a:3}/{b:<3} {k}")
if mode == "--save":
    open(BASE, "w").write("\n".join(sorted(passing)) + "\n")
    print(f"spec-blocks: saved {len(passing)} checking block(s)")
elif mode == "--ratchet":
    for b in broke:
        print(f"spec-blocks: no longer checks: {b}")
    sys.exit(1 if broke else 0)
else:
    for e, n in errs.most_common(int(sys.argv[1]) if len(sys.argv) > 1 else 10):
        print(f"  {e} x{n}  e.g. {first[e]}")
