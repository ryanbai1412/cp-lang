#!/usr/bin/env python3
"""A g++ that reuses its last output for the same source and flags.

scripts/check.sh puts this first on PATH as `g++` for the fast gate. Most
commits leave most emitted C++ unchanged, and the test suites compile the same
programs again on every run, so a hit copies the binary instead of compiling.
The key is the real compiler's identity, every argument except the output
path, and the bytes of each source file (or of stdin). Only successful
compiles are kept; anything the cache can't key runs the real compiler.
"""

import hashlib
import os
import shutil
import subprocess
import sys

SOURCE_EXTS = (".cpp", ".cc", ".cxx", ".c++", ".C")


def real_compiler():
    here = os.path.dirname(os.path.abspath(__file__))
    shim_dir = os.environ.get("CPL_CXX_SHIM_DIR", "")
    for d in os.environ.get("PATH", "").split(os.pathsep):
        if not d or os.path.abspath(d) in (here, os.path.abspath(shim_dir)):
            continue
        cand = os.path.join(d, "g++")
        if os.access(cand, os.X_OK) and os.path.realpath(cand) != os.path.realpath(sys.argv[0]):
            return cand
    sys.exit("cxx-cache: no g++ on PATH besides the cache")


def main():
    gxx = real_compiler()
    args = sys.argv[1:]
    cache = os.environ.get("CPL_CXX_CACHE_DIR")
    if not cache or "-o" not in args or args.index("-o") + 1 >= len(args):
        os.execv(gxx, [gxx] + args)
    out_i = args.index("-o") + 1
    out = args[out_i]
    stdin_bytes = None
    h = hashlib.sha256()
    st = os.stat(os.path.realpath(gxx))
    h.update(f"{os.path.realpath(gxx)}\0{st.st_size}\0{st.st_mtime_ns}\0".encode())
    h.update(os.environ.get("CPATH", "").encode() + b"\0")
    sources = 0
    for i, a in enumerate(args):
        if i == out_i:
            h.update(b"<out>\0")
            continue
        if a == "-":
            stdin_bytes = sys.stdin.buffer.read()
            h.update(b"<stdin>" + hashlib.sha256(stdin_bytes).digest() + b"\0")
            sources += 1
        elif a.endswith(SOURCE_EXTS) and os.path.isfile(a):
            with open(a, "rb") as f:
                h.update(b"<src>" + hashlib.sha256(f.read()).digest() + b"\0")
            sources += 1
        elif not a.startswith("-") and os.path.exists(a):
            # An input the key can't see inside (an object, a header dir).
            sources = -1
            break
        else:
            h.update(a.encode() + b"\0")
    if sources <= 0:
        if stdin_bytes is not None:
            r = subprocess.run([gxx] + args, input=stdin_bytes)
            sys.exit(r.returncode)
        os.execv(gxx, [gxx] + args)
    key = h.hexdigest()
    entry = os.path.join(cache, key[:2], key)
    if os.path.isfile(entry + ".bin"):
        tmp = f"{out}.cxx-cache-{os.getpid()}"
        shutil.copyfile(entry + ".bin", tmp)
        os.chmod(tmp, 0o755)
        os.replace(tmp, out)
        try:
            with open(entry + ".err", "rb") as f:
                sys.stderr.buffer.write(f.read())
        except FileNotFoundError:
            pass
        os.utime(entry + ".bin")
        return 0
    r = subprocess.run([gxx] + args, input=stdin_bytes, stderr=subprocess.PIPE)
    sys.stderr.buffer.write(r.stderr)
    if r.returncode == 0 and os.path.isfile(out):
        os.makedirs(os.path.dirname(entry), exist_ok=True)
        tmp = f"{entry}.{os.getpid()}"
        shutil.copyfile(out, tmp + ".bin")
        with open(tmp + ".err", "wb") as f:
            f.write(r.stderr)
        os.replace(tmp + ".err", entry + ".err")
        os.replace(tmp + ".bin", entry + ".bin")
    return r.returncode


if __name__ == "__main__":
    sys.exit(main())
