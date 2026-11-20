#!/usr/bin/env python3
"""Regression tests for the P0 silent-wrong-answer bugs (see tests/p0/README.md). Run from proto/."""
import os, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.dirname(HERE)
P0 = os.path.join(HERE, "p0")
CP = [sys.executable, os.path.join(PROTO, "cp.py")]
CXX = ["g++", "-std=c++17", "-O1", "-fsanitize=address,undefined", "-fno-sanitize-recover=undefined"]
failed = []

def check(name, ok, detail=""):
    if not ok:
        failed.append(name)
        print(f"P0 REGRESSION: {name}: {detail[:300]}")

def compiled(name, flags=("-DLOCAL",)):
    """The output of tests/p0/<name>.cp built by cpc and run under ASan + UBSan."""
    with tempfile.TemporaryDirectory() as tmp:
        shutil.copy(os.path.join(P0, name + ".cp"), os.path.join(tmp, "t.cp"))
        r = subprocess.run(CP + ["build", os.path.join(tmp, "t.cp")], capture_output=True, text=True)
        if r.returncode:
            return "build failed: " + r.stderr[-200:]
        r = subprocess.run(CXX + list(flags) + ["-o", os.path.join(tmp, "t"), os.path.join(tmp, "t.cpp")], capture_output=True, text=True)
        if r.returncode:
            return "g++ failed: " + r.stderr[:200]
        inp = os.path.join(P0, name + ".in")
        r = subprocess.run([os.path.join(tmp, "t")], input=open(inp).read() if os.path.exists(inp) else "",
                           capture_output=True, text=True, timeout=20)
        return r.stdout + r.stderr

def interpreted(name):
    inp = os.path.join(P0, name + ".in")
    r = subprocess.run(CP + ["lib-run", os.path.join(P0, name + ".cp")], input=open(inp).read() if os.path.exists(inp) else "",
                       capture_output=True, text=True, timeout=20, cwd=PROTO)
    return r.stdout + r.stderr

def stress(case, *args, cmd="stress"):
    with tempfile.TemporaryDirectory() as tmp:
        for f in os.listdir(os.path.join(P0, "stress")):
            if f.startswith(case):
                shutil.copy(os.path.join(P0, "stress", f), tmp)
        r = subprocess.run(CP + [cmd, os.path.join(tmp, case + ".cp"), os.path.join(tmp, case + "_ref.cpp"), *args],
                           capture_output=True, text=True, timeout=300)
        return r.returncode, r.stdout + r.stderr

def interactive_pipe(cmd):
    """Run an interactive program over live pipes: each reply is sent ONLY after the previous prompt arrives."""
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, cwd=PROTO)
    try:
        p.stdin.write("5\n"); p.stdin.flush()
        l1 = p.stdout.readline().strip()
        if l1 != "6":
            return f"expected '6', got {l1!r}"
        p.stdin.write("10\n"); p.stdin.flush()
        l2 = p.stdout.readline().strip()
        if l2 != "20":
            return f"expected '20', got {l2!r}"
        p.stdin.write("7\n"); p.stdin.flush()
        l3 = p.stdout.readline().strip()
        if l3 != "17":
            return f"expected '17', got {l3!r}"
        p.stdin.close()
        rc = p.wait(timeout=5)
        err = p.stderr.read()
        return "OK" if rc == 0 and not err else f"rc={rc} err={err}"
    except Exception as e:
        p.kill()
        return str(e)

want = lambda name: open(os.path.join(P0, name + ".out")).read()
for name in ("shift64", "value_copy", "array_literal", "brute_not_main", "float_format", "index_assign_grow"):
    got = compiled(name)
    check(f"{name} (cpc, -DLOCAL)", got == want(name), got)
for name in ("shift64", "brute_not_main", "index_assign_grow"): # the judge build (no -DLOCAL) must agree
    got = compiled(name, flags=())
    check(f"{name} (cpc, judge build)", got == want(name), got)
for name in ("shift64", "value_copy_cpi", "array_literal_cpi", "float_format", "index_assign_grow"):
    got = interpreted(name)
    check(f"{name} (cpi)", got == want(name), got)

with tempfile.TemporaryDirectory() as tmp:
    shutil.copy(os.path.join(P0, "interactive_flush.cp"), os.path.join(tmp, "t.cp"))
    r = subprocess.run(CP + ["build", os.path.join(tmp, "t.cp")], capture_output=True, text=True)
    if r.returncode:
        check("interactive_flush (cpc)", False, r.stderr[-200:])
    else:
        r = subprocess.run(CXX + ["-DLOCAL", "-o", os.path.join(tmp, "t"), os.path.join(tmp, "t.cpp")], capture_output=True, text=True)
        if r.returncode:
            check("interactive_flush (cpc)", False, r.stderr[:200])
        else:
            res = interactive_pipe([os.path.join(tmp, "t")])
            check("interactive_flush (cpc)", res == "OK", res)
res = interactive_pipe(CP + ["lib-run", os.path.join(P0, "interactive_flush.cp")])
check("interactive_flush (cpi)", res == "OK", res)

code, out = stress("late_bug")
check("stress finds a bug that needs n >= 12", code == 2 and "MISMATCH" in out, out)
code, out = stress("late_bug", "--max-size", "8")
check("stress reports its coverage", code == 0 and "coverage: sizes 1..8" in out and "compared by lines" in out
      and "distinct reference outputs" in out, out)
code, out = stress("late_bug", cmd="locate")
check("locate finds a bug that needs n >= 12", "smallest failing input" in out, out)
code, out = stress("line_breaks")
check("stress compares line by line", code == 2 and "MISMATCH" in out, out)
code, out = stress("line_breaks", "--tokens")
check("stress --tokens compares tokens", code == 0 and "compared by tokens" in out, out)
code, out = stress("all_rejected")
check("an all-rejected stress run is not OK", code == 3 and "INCONCLUSIVE" in out and "stress OK" not in out, out)

# Synthesis #2 (research/language-architecture-synthesis-2.md): the seven kernel rules and the format specifiers.
# Synthesis #3 (research/language-architecture-synthesis-3.md): multi-binding, `[expr]T`, local `!` narrowing,
# optional-accepting `.get(?I)` + trapping `D.from_idx`, `const struct` + `if const` + erased `()` fields.
# Every tests/p0/s2_*.cp and s3_*.cp must print its .out on both paths: built by cpc under ASan + UBSan, and run by cpi
# (`*_cpc.cp` tests cpc-only containers like Heap/Deque; `*_trap.cp` must trap at runtime on both paths).
S2_S3 = sorted(f[:-3] for f in os.listdir(P0) if (f.startswith("s2_") or f.startswith("s3_")) and f.endswith(".cp"))
for name in S2_S3:
    if name.endswith("_trap"):      # a runtime error on both paths, never a wrong value
        got = compiled(name)
        check("cpc " + name, "line " in got and "g++ failed" not in got and "build failed" not in got, got)
        got = interpreted(name)
        check("cpi " + name, "error:" in got, got)
        continue
    want = open(os.path.join(P0, name + ".out")).read()
    got = compiled(name)
    check("cpc " + name, got == want, got)
    if not name.endswith("_cpc"):
        got = interpreted(name)
        check("cpi " + name, got == want, got)

# Verify `if const` field erasure in emitted C++ for `s3_const_config.cp`: the LITE specialization omits `up` and `tin`.
with tempfile.TemporaryDirectory() as tmp:
    shutil.copy(os.path.join(P0, "s3_const_config.cp"), os.path.join(tmp, "t.cp"))
    r = subprocess.run(CP + ["build", os.path.join(tmp, "t.cp")], capture_output=True, text=True)
    if r.returncode:
        check("cpc s3_const_config erasure", False, r.stderr[-200:])
    else:
        cpp = open(os.path.join(tmp, "t.cpp")).read()
        # Look for two distinct struct declarations for Rooted: the full one has `up` & `tin`, the lite one has only `depth`.
        import re as _re
        structs = _re.findall(r"struct\s+\w+\s*\{[^\n]*\bdepth;[^\n]*", cpp)
        has_full = any("up;" in body and "tin;" in body and "depth;" in body for body in structs)
        has_lite = any("depth;" in body and "up;" not in body and "tin;" not in body for body in structs)
        check("cpc s3_const_config erasure", has_full and has_lite, f"structs={structs}")

# The spellings Synthesis #2 rejects must be rejected by the interpreter too, where it can tell without static types
# (run_all.sh checks every tests/bad/*.cp against cpc).
for name in ("s2_read_no_type", "s2_read_angle", "s2_fmt_align", "s2_fmt_nested_width", "s2_fmt_float_width_only",
             "s2_fmt_hex_float", "s2_int_of_f64", "s2_char_of_digit", "s2_for_mut", "s2_array_default_literal"):
    r = subprocess.run(CP + ["lib-run", os.path.join(HERE, "bad", name + ".cp")], input="", capture_output=True, text=True,
                       timeout=20, cwd=PROTO)
    check("cpi rejects " + name, "error:" in r.stdout + r.stderr, r.stdout + r.stderr)

print(f"p0 regressions: {'all passed' if not failed else str(len(failed)) + ' FAILED'}")
sys.exit(1 if failed else 0)
