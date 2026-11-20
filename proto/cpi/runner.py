"""`cp.py test`, `cp.py doc` and `cp.py lib-run`: the library commands."""
import os
import sys
import time

from .interp import Interp
from .values import CpError


def cp_files(path):
    if os.path.isfile(path):
        return [path]
    out = []
    for d, _, fs in sorted(os.walk(path)):
        out += [os.path.join(d, f) for f in sorted(fs) if f.endswith(".cp")]
    # a solution (a file that reads input) is not a library module; only modules carry tests
    return [f for f in out if "@test" in open(f).read()]


def run_tests(paths, only=None):
    """Run every `@test fn` in the given files or folders. Returns the number of failures."""
    it = Interp()
    passed = failed = 0
    t0 = time.time()
    for path in [f for p in paths for f in cp_files(p)]:
        rel = os.path.relpath(path)
        try:
            mod = it.load(path)
        except CpError as e:
            print(f"FAIL {rel}: {e}")
            failed += 1
            continue
        for t in mod.tests:
            if only and only not in t.name:
                continue
            it.rng.seed(1)
            it.mod, it.depth = mod, 0
            try:
                it.call(mod.env.vars[t.name], [], None, t)
                passed += 1
                print(f"ok   {rel}::{t.name}")
            except CpError as e:
                failed += 1
                print(f"FAIL {rel}::{t.name}\n     {e}")
            except RecursionError:
                failed += 1
                print(f"FAIL {rel}::{t.name}\n     the interpreter ran out of stack")
    print(f"{passed} passed, {failed} failed in {time.time() - t0:.1f}s")
    return failed


def run_file(path, text):
    """Run a solution through the interpreter, reading `text` as its input."""
    it = Interp(stdin_tokens=text.split())
    try:
        it.load(path)
    except CpError as e:
        sys.exit(f"error: {e}")


def signature(src, line):
    """The header of the declaration at `line`, without its body."""
    s = src[line - 1].strip()
    for stop in (" = ", ":"):
        if s.startswith(("pub fn", "fn", "pub @get fn")) and stop == " = " and stop in s:
            s = s.split(stop)[0]
    return s[:-1] if s.endswith(":") else s


def doc(path):
    """Markdown reference for the `pub` API of a library, from its `///` comments."""
    from .parser import LibParser
    out = []
    for f in cp_files(path):
        src = open(f).read().split("\n")
        tree = LibParser("\n".join(src)).program()
        head = []
        for ln in src:
            if not ln.startswith("//!"):
                break
            head.append(ln[3:].strip())
        items = []
        for s in tree.body:
            if not getattr(s, "pub", False):
                continue
            items.append((s, "###"))
            if s.kind == "struct":
                items += [(m, "####") for m in s.methods if getattr(m, "pub", False)]
        if not items:
            continue
        out.append(f"## {os.path.relpath(f, path)}\n")
        if head:
            out.append("\n".join(head) + "\n")
        for s, h in items:
            out.append(f"{h} `{signature(src, s.line)}`\n")
            if getattr(s, "doc", ""):
                out.append(s.doc + "\n")
    return "\n".join(out)
