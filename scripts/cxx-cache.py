#!/usr/bin/env python3
"""A g++ that reuses its last output for the same source and flags.
Rule: handbook/guidelines/ci-checklist.md#cir16.

scripts/check.sh puts this first on PATH as `g++` for the fast gate. Most
commits leave most emitted C++ unchanged, and the test suites compile the same
programs again on every run, so a hit copies the binary instead of compiling.
The key is the real compiler's identity, every argument except the output
path, the bytes of each source file (or of stdin), and a fingerprint of every
system header and library directory the compiler searches. Only successful
compiles are kept, entries appear by atomic rename, and the least recently used
are evicted past CPL_CXX_CACHE_MAX_BYTES. Anything the cache can't key (a
quoted include, an include or library flag, a search-path variable) runs the
real compiler.
"""

import hashlib
import os
import shutil
import subprocess
import sys
import tempfile

SOURCE_EXTS = (".cpp", ".cc", ".cxx", ".c++", ".C")
SEARCH_FLAGS = ("-I", "-isystem", "-iquote", "-idirafter", "-iprefix", "-include",
                "-imacros", "-isysroot", "--sysroot", "-B", "-L", "-specs", "-nostdinc", "@")
SEARCH_ENV = ("CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH", "OBJC_INCLUDE_PATH",
              "LIBRARY_PATH", "GCC_EXEC_PREFIX", "COMPILER_PATH")
DEFAULT_MAX_BYTES = 2 << 30


def real_compiler():
    """The first g++ on PATH that is not a cache shim. Each choice's directory
    goes into CPL_CXX_SHIM_SKIP for the compiler's children, so a wrapper that
    runs `g++` from PATH again, or a second checkout's shim, moves on to the
    next g++ instead of looping back through the shim."""
    here = os.path.dirname(os.path.abspath(__file__))
    shim_dir = os.environ.get("CPL_CXX_SHIM_DIR", "")
    skip = {os.path.abspath(d) for d in os.environ.get("CPL_CXX_SHIM_SKIP", "").split(os.pathsep) if d}
    skip |= {here, os.path.abspath(shim_dir)}
    for d in os.environ.get("PATH", "").split(os.pathsep):
        if not d or os.path.abspath(d) in skip:
            continue
        cand = os.path.join(d, "g++")
        if not os.access(cand, os.X_OK) or os.path.basename(os.path.realpath(cand)) == "cxx-cache.py":
            continue
        os.environ["CPL_CXX_SHIM_SKIP"] = os.pathsep.join(sorted(skip | {os.path.abspath(d)}))
        return cand
    sys.exit("cxx-cache: no g++ on PATH besides the cache")


def cpu_key():
    """The CPU's model and feature flags, for builds that tune to it
    (`-march=native`): a binary from another CPU can die on SIGILL."""
    try:
        with open(os.environ.get("CPL_CXX_CPU_INFO", "/proc/cpuinfo")) as f:
            lines = f.read().splitlines()
    except OSError:
        return None
    keep = [l for l in lines if l.split(":")[0].strip() in ("model name", "flags", "Features")]
    return "\n".join(sorted(set(keep))) or None


def system_key(gxx):
    """Every file under the compiler's include dirs and in its library dirs,
    by path, size and mtime. None if the compiler won't name its dirs."""
    try:
        v = subprocess.run([gxx, "-xc++", "-E", "-v", "-"], input=b"",
                           capture_output=True, check=True).stderr.decode()
        libs = subprocess.run([gxx, "-print-search-dirs"], capture_output=True,
                              check=True).stdout.decode()
    except (OSError, subprocess.CalledProcessError):
        return None
    lines = v.splitlines()
    try:
        start = lines.index("#include <...> search starts here:") + 1
        end = lines.index("End of search list.")
    except ValueError:
        return None
    includes = [l.strip() for l in lines[start:end]]
    lib_dirs = []
    for l in libs.splitlines():
        if l.startswith("libraries: ="):
            lib_dirs = [d for d in l[len("libraries: ="):].split(os.pathsep) if d]
    if not includes or not lib_dirs:
        return None
    h = hashlib.sha256()

    def add(path):
        try:
            st = os.stat(path)
        except OSError:
            return
        h.update(f"{path}\0{st.st_size}\0{st.st_mtime_ns}\0".encode())

    for d in includes:
        for r, ds, fs in os.walk(d):
            ds.sort()
            add(r)
            for f in sorted(fs):
                add(os.path.join(r, f))
    for d in lib_dirs:
        d = os.path.normpath(d)
        add(d)
        try:
            names = sorted(os.listdir(d))
        except OSError:
            continue
        for f in names:
            add(os.path.join(d, f))
    return h.hexdigest()


def evict(cache, max_bytes):
    entries = []
    total = 0
    for r, _, fs in os.walk(cache):
        for f in fs:
            path = os.path.join(r, f)
            try:
                st = os.stat(path)
            except OSError:
                continue
            total += st.st_size
            if f.endswith(".bin"):
                entries.append((st.st_mtime_ns, path))
    if total <= max_bytes:
        return
    for _, path in sorted(entries):
        for p in (path, path[:-4] + ".err"):
            try:
                total -= os.stat(p).st_size
                os.remove(p)
            except OSError:
                pass
        if total <= max_bytes:
            return


def main():
    gxx = real_compiler()
    args = sys.argv[1:]
    if args == ["--cpl-system-key"]:
        key = system_key(gxx)
        if key is None:
            return 1
        print(key)
        return 0
    cache = os.environ.get("CPL_CXX_CACHE_DIR")
    if (not cache or "-o" not in args or args.index("-o") + 1 >= len(args)
            or any(os.environ.get(v) for v in SEARCH_ENV)
            or any(a.startswith(SEARCH_FLAGS) for a in args)):
        os.execv(gxx, [gxx] + args)
    sys_key = os.environ.get("CPL_CXX_SYSTEM_KEY") or system_key(gxx)
    if not sys_key:
        os.execv(gxx, [gxx] + args)
    out_i = args.index("-o") + 1
    out = args[out_i]
    stdin_bytes = None
    h = hashlib.sha256()
    st = os.stat(os.path.realpath(gxx))
    h.update(f"{os.path.realpath(gxx)}\0{st.st_size}\0{st.st_mtime_ns}\0".encode())
    h.update(sys_key.encode() + b"\0")
    if any(a.endswith("=native") for a in args):
        cpu = cpu_key()
        if cpu is None:
            os.execv(gxx, [gxx] + args)
        h.update(cpu.encode() + b"\0")
    sources = 0
    for i, a in enumerate(args):
        if i == out_i:
            h.update(b"<out>\0")
            continue
        if a == "-" or (a.endswith(SOURCE_EXTS) and os.path.isfile(a)):
            if a == "-":
                stdin_bytes = body = sys.stdin.buffer.read()
            else:
                with open(a, "rb") as f:
                    body = f.read()
            if b'#include "' in body or b"#include\"" in body:
                # A local header the key can't see.
                sources = -1
                break
            h.update(b"<src>" + hashlib.sha256(body).digest() + b"\0")
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
        try:
            with open(entry + ".err", "rb") as f:
                err = f.read()
            shutil.copyfile(entry + ".bin", tmp)
            os.chmod(tmp, 0o755)
            os.replace(tmp, out)
            os.utime(entry + ".bin")
            sys.stderr.buffer.write(err)
            return 0
        except OSError:
            # Evicted by a concurrent run between the check and the copy.
            try:
                os.remove(tmp)
            except OSError:
                pass
    r = subprocess.run([gxx] + args, input=stdin_bytes, stderr=subprocess.PIPE)
    sys.stderr.buffer.write(r.stderr)
    if r.returncode == 0 and os.path.isfile(out):
        os.makedirs(os.path.dirname(entry), exist_ok=True)
        fd, tmp = tempfile.mkstemp(prefix=key + ".", suffix=".tmp", dir=os.path.dirname(entry))
        os.close(fd)
        shutil.copyfile(out, tmp)
        with open(tmp + ".err", "wb") as f:
            f.write(r.stderr)
        os.replace(tmp + ".err", entry + ".err")
        os.replace(tmp, entry + ".bin")
        max_bytes = int(os.environ.get("CPL_CXX_CACHE_MAX_BYTES") or DEFAULT_MAX_BYTES)
        evict(cache, max_bytes)
    return r.returncode


if __name__ == "__main__":
    sys.exit(main())
