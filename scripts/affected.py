#!/usr/bin/env python3
"""Pick what the commit tier of the gate runs for a set of changed paths.
Rule: handbook/guidelines/ci-checklist.md#cir3.

    affected.py [--json] [path ...]     default: the staged paths
    affected.py [--json] --base REV     the paths changed between REV and HEAD
    affected.py --run-slices NAME ...   run touched-path slices; one over 20 s warns its owner
    affected.py --run-slices all        every slice (CI's wall-clock job reports the slow ones)
    affected.py --run-slices --base REV NAME ...
                                        fail only on the reds REV doesn't have (land.sh, ci/R40)

A path under a crate selects that crate and, through nextest's `rdeps()`,
every crate that depends on it. Prose selects nothing. Any path this file
does not know selects the whole fast gate, a checking script that
scripts/ci.py runs selects its own step, so a new kind of input can only
make the commit tier slower, never let it miss a test. The land tier
(`scripts/check.sh --land`) selects the same way over a train's diff; CI's
`--fast` on every main tip always runs everything.
"""

import fnmatch
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ci  # noqa: E402
import corpus  # noqa: E402

PACKAGES = corpus.rel(corpus.PROBLEMS)

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Read by no crate, test or script check (scripts/check_cache.py's excludes).
PROSE = ["AGENTS.md", "REPO.md", "decisions.md", "worklog.md", "research/*",
         "plan/*", "corpus/submissions/*", "corpus/statements/*", "archive/*",
         ".github/*", "LICENSE*", "*.png"]
# Inputs of every crate: build settings and the std library cpl-check embeds.
EVERY_CRATE = ["Cargo.toml", "Cargo.lock", "rust-toolchain*", ".cargo/*",
               ".config/nextest.toml", "std/*", "clippy.toml", "rustfmt.toml"]
SITE = ["web/*", "docs/*"]
# Paths that remove or rename a std spelling the site's snippets may still use.
SITE_SPELLINGS = ["crates/cpl-check/src/check/removed.rs", "std/prelude.cp"]
# Data files and the nextest filter of the tests that read them.
DATA_TESTS = [(f"{PACKAGES}/*/{leaf}", "package(cpl-lint) & test(/corpus_warnings/)")
              for leaf in ("spec.cp", "port.cp")] + [
    # A record's code is checked against main's at the rebased tip, so two lanes'
    # records that take one code can't both land.
    (f"spec/{d}/*/lint.toml", "package(cpl-lint) & test(/record_codes_are_unique/)")
    for d in ("lints", "checker-lints")]
# Editor files a crate embeds or its tests read; the rest are CI's editor jobs'.
# Programs no test reads but that must still pass `cpl check`.
CHECK_ONLY = ["research/acceptance/solutions/*.cp", f"{PACKAGES}/*/perf.cp"]
# Package files the commit tier leaves to the corpus audit; CI runs the rest.
AUDIT_ONLY = [f"{PACKAGES}/*/{leaf}" for leaf in ("gen.py", "tests.toml", "samples/*", "orig.cpp", "meta.json")]
# Owner-named tests a touched path runs ahead of the rest of the commit tier,
# each under SLICE_LIMIT_S warm (ci/R39): name, owner, paths, nextest
# `binary_id[ test]` entries.
SLICE_LIMIT_S = 20
SLICES = [
    ("vm-trace", "consumers (trace)",
     ["std/*", "crates/cpl-syntax/*", "crates/cpl-diag/*", "crates/cpl-check/*",
      "crates/cpl-ir/*", "crates/cpl-interp/*",
      "crates/cpl-cli/tests/programs/*",
      "crates/cpl-cli/tests/vm_fallbacks*"],
     ["cpl-cli::trace_goldens", "cpl-cli::vm_fallbacks"]),
    ("corpus-warnings", "consumers",
     ["std/*", f"{PACKAGES}/*/spec.cp", "corpus/newcomer/*", "crates/cpl-lint/*", "crates/cpl-check/*"],
     ["cpl-lint corpus_tests::corpus_warnings_only_go_down"]),
    ("ports-check", "content (stdlib)", ["std/*"],
     ["cpl-cli::corpus the_libraries_ports_and_passing_tests_check_clean"]),
    ("browser-std", "content (stdlib)", ["std/*", "crates/cpl-check/src/std_bundle.rs",
                                         "crates/cpl-check/src/std_files.rs"],
     ["cpl-check modules::tests::std_in_the_browser_build_is_within_its_gzipped_cap"]),
    ("silent-unknowns", "compiler (check)",
     ["std/*", f"{PACKAGES}/*.cp", "spec/conformance/*.cp", "crates/cpl-check/*",
      "crates/cpl-conform/silent_unknowns.txt"],
     ["cpl-conform::silent_unknowns"]),
    ("copies", "perf (codegen)", [f"{PACKAGES}/*.cp", "spec/conformance/*.cp", "std/*", "crates/cpl-codegen/*",
                                  "crates/cpl-codegen/src/cpp_counts.rs",
                                  "crates/cpl-cli/tests/support/result_cache.rs"],
     ["cpl-cli::read_only_copies"]),
    ("readable-inputs", "debugger (stress)",
     [f"{PACKAGES}/*/spec.cp", "spec/conformance/*.cp", "crates/cpl-cli/tests/programs/*", "crates/cpl-stress/*"],
     ["cpl-stress::readable_shapes"]),
    # What an editor reads: the server's answers, the JSON and text every command prints,
    # and the settings each extension declares, so a change to any of them can't land unrun.
    ("editor-contract", "tools",
     ["std/*", "editors/*", "crates/cpl-lsp/*", "crates/cpl-cli/src/*", "crates/cpl-diag/src/view.rs",
      "spec/tools/cli.md", "spec/tools/lsp.md"],
     ["cpl-lsp::import", "cpl-lsp::fixtures",
      "cpl-lsp::std_items every_builtin_member_hovers_signs_and_completes_with_its_parameters",
      "cpl-cli::json_contract", "cpl-cli::output_pins", "cpl-cli::vscode_sync"]),
]
# A sharded slice's binary has a test `shard_NN` per shard of its programs, NN being the
# FNV-1a hash of the program's path mod the shard count (shard_of in
# crates/cpl-cli/tests/read_only_copies.rs), so a change to programs alone runs their
# shards and the binary's other tests: the slice `copies@3,7`.
SHARDED = {"copies": (16, [f"{PACKAGES}/*/*.cp", "spec/conformance/*/*.cp"])}
# Globs name the data and crates a slice's tests run; slice_globs adds each test's
# source and the workspace crates it names. scripts/test_affected.py fails on a
# path literal in a test that its slice's globs miss.


def test_source(test):
    """`cpl-cli::corpus name` -> crates/cpl-cli/tests/corpus.rs; `cpl-lint mod::name` -> src/mod.rs."""
    binary, _, name = test.partition(" ")
    crate, _, target = binary.partition("::")
    return f"crates/{crate}/tests/{target}.rs" if target else f"crates/{crate}/src/{name.split('::')[0]}.rs"


def slice_globs(globs, tests):
    out = list(globs)
    for t in tests:
        src = test_source(t)
        own = src.split("/")[1]
        try:
            text = (Path(ROOT) / src).read_text(encoding="utf-8")
        except OSError:
            text = ""
        # Its own crate's change already runs it, through nextest's rdeps.
        named = set()
        for d in re.findall(r"\bcpl_([a-z_]+)::", text):
            named |= crate_deps("cpl-" + d.replace("_", "-"))
        # A test that runs a binary runs every crate the binary links: 41ffcffe5 changed
        # cpl-codegen's output and landed with cpl-cli's output_pins unrun.
        for b in re.findall(r"CARGO_BIN_EXE_([a-z_-]+)", text):
            named |= crate_deps(BIN_CRATES.get(b, b))
        out += [src] + [f"crates/{c}/*" for c in sorted(named - {own})]
    return out


BIN_CRATES = {"cpl": "cpl-cli"}


def crate_deps(crate, seen=None):
    """`crate` and the workspace crates it depends on, transitively (build and dev deps aside)."""
    seen = set() if seen is None else seen
    if crate in seen:
        return seen
    seen.add(crate)
    try:
        toml = (Path(ROOT) / "crates" / crate / "Cargo.toml").read_text(encoding="utf-8")
    except OSError:
        return seen
    deps = re.split(r"^\[(?:dev-|build-)dependencies\]", toml, flags=re.M)[0]
    for d in re.findall(r"^(cpl-[a-z-]+)\s*[.=]", deps, flags=re.M):
        crate_deps(d, seen)
    return seen
EDITOR_CRATES = {"editors/neovim/": "cpl-cli", "editors/vscode/": "cpl-cli",
                 "editors/jetbrains/": "cpl-cli", "editors/tree-sitter-cp/": "cpl-cli",
                 "editors/lsp-matrix.json": "cpl-lsp"}


def matches(path, patterns):
    return any(fnmatch.fnmatch(path, p) for p in patterns)


def crate_dirs():
    """Map each workspace crate's directory to its package name."""
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    dirs = {}
    for pkg in json.loads(out)["packages"]:
        rel = os.path.relpath(os.path.dirname(pkg["manifest_path"]), ROOT)
        dirs[rel.replace(os.sep, "/") + "/"] = pkg["name"]
    return dirs


def script_test(path):
    """The unit test file that covers a script, or None."""
    name = os.path.basename(path)
    if name in ("land.sh", "check.sh", "check_cache.py", "gate-env.sh"):
        return "scripts/test_check_cache.py"
    if path.startswith("scripts/release/"):
        return "scripts/test_release.py"
    if name == "new_reds.py":
        return "scripts/test_land.py"
    stem = os.path.splitext(name)[0].replace("-", "_")
    test = f"scripts/test_{stem.removeprefix('test_')}.py"
    return test if os.path.exists(os.path.join(ROOT, test)) else None


# What runs scripts locally; a script none of them names runs only in CI.
LOCAL_RUNNERS = ["scripts/check.sh", "scripts/land.sh", "justfile", ".githooks/pre-commit",
                 ".githooks/pre-push", ".githooks/commit-msg"]


def ci_only(path):
    """A script only a workflow runs: the commit tier has nothing to test."""
    name = os.path.basename(path)
    for runner in LOCAL_RUNNERS:
        try:
            with open(os.path.join(ROOT, runner), encoding="utf-8") as f:
                if name in f.read():
                    return False
        except FileNotFoundError:
            pass
    return path.endswith(".sh")


def shard_of(rel, shards):
    h = 0xcbf29ce484222325
    for b in rel.encode():
        h = ((h ^ b) * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return h % shards


def sharded(name, globs, tests, paths):
    """`name`, or `name@<its shards>` when each path it maps to is one of its sharded programs."""
    if name not in SHARDED:
        return name
    shards, programs = SHARDED[name]
    hit = [p for p in paths if matches(p, slice_globs(globs, tests))]
    if not all(any(fnmatch.fnmatch(p, g) and p.count("/") == g.count("/") for g in programs) for p in hit):
        return name
    return f"{name}@{','.join(str(k) for k in sorted({shard_of(p, shards) for p in hit}))}"


def base(names):
    return [n.partition("@")[0] for n in names]


def slice_filter(names):
    picked = {}
    for n in names:
        name, _, shards = n.partition("@")
        picked.setdefault(name, set()).update(shards.split(",") if shards else ["all"])
    parts = []
    for name, _, _, tests in SLICES:
        if name in picked:
            for t in tests:
                binary, _, test = t.partition(" ")
                part = f"(binary_id({binary}) & test(={test}))" if test else f"binary_id({binary})"
                if "all" not in picked[name]:
                    some = " | ".join(f"test(=shard_{int(k):02})" for k in sorted(picked[name], key=int))
                    part = f"({part} & (not test(/^shard_/) | {some}))"
                parts.append(part)
    return " | ".join(parts)


def slice_times(output, names):
    """Each named slice's time in a nextest run's output: its slowest test (they run in parallel)."""
    times = {n: 0.0 for n in names}
    for m in re.finditer(r"^\s*(?:PASS|FAIL|TIMEOUT|SIGSEGV|SIGABRT) \[\s*([\d.]+)s\] (?:\(\d+/\d+\) )?(\S+) (\S+)",
                         output, re.M):
        secs, binary, test = float(m[1]), m[2], m[3]
        for name, _, _, tests in SLICES:
            if name in times and any(t in (binary, f"{binary} {test}") for t in tests):
                times[name] = max(times[name], secs)
    return times


def build_slices(names):
    """Compiles the slices' test binaries: the commit hook times this apart from
    the run, since a cold build after a crate change is slow compiling, not a slow test."""
    unknown = [n for n in base(names) if n not in {s[0] for s in SLICES}]
    if unknown:
        print(f"affected: unknown slice {' '.join(unknown)}; one of {', '.join(s[0] for s in SLICES)}")
        return 2
    return subprocess.run(["cargo", "nextest", "run", "--workspace", "--no-run", "-E", slice_filter(names)],
                          cwd=ROOT).returncode


def run_slices(names, reds_base=None):
    if names == ["all"]:
        names = [s[0] for s in SLICES]
    unknown = [n for n in base(names) if n not in {s[0] for s in SLICES}]
    if unknown:
        print(f"affected: unknown slice {' '.join(unknown)}; one of {', '.join(s[0] for s in SLICES)}")
        return 2
    expr = slice_filter(names)
    built = build_slices(names)
    if built:
        return built
    cmd = ["cargo", "nextest", "run", "--workspace", "--no-fail-fast", "--status-level", "pass", "-E", expr]
    # A slice red at the base too is main's, reported with its owner (scripts/new_reds.py).
    if reds_base:
        cmd = [sys.executable, os.path.join(ROOT, "scripts", "new_reds.py"), "--base", reds_base, "--", *cmd]
    run = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    sys.stdout.write(run.stdout + run.stderr)
    owners = {s[0]: s[1] for s in SLICES}
    slow = {n: t for n, t in slice_times(run.stdout + run.stderr, base(names)).items() if t > SLICE_LIMIT_S}
    # A slow slice is its owner's to speed up, never a reason to refuse another lane's
    # commit: it warns and is logged for the owner, and only a failing test fails.
    log = os.path.join(ROOT, ".git", "cpl-slow-slices.log") if os.path.isdir(os.path.join(ROOT, ".git")) else None
    for name, secs in slow.items():
        line = (f"affected: WARNING: slice {name} took {secs:.1f}s, over the {SLICE_LIMIT_S} s limit; "
                f"its owner ({owners[name]}) splits it or makes it faster")
        print(f"::warning::{line}" if os.environ.get("GITHUB_ACTIONS") else line)
        if log:
            with open(log, "a", encoding="utf-8") as f:
                f.write(f"{int(time.time())} {name} {secs:.1f}s {owners[name]}\n")
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as f:
            f.write(f"slow slices: {len(slow)} of {len(names)} over {SLICE_LIMIT_S} s"
                    + "".join(f"\n- {n} {t:.1f}s ({owners[n]})" for n, t in sorted(slow.items())) + "\n")
    return run.returncode


# What `cpl build` on the old pipeline is built from: a change here must leave its output
# byte-identical (scripts/old-pipeline-same.py, the freeze's behaviour half).
OLD_PIPELINE_INPUTS = ("crates/cpl-cli/", "crates/cpl-check/", "crates/cpl-ir/", "crates/cpl-interp/",
                       "crates/cpl-codegen/", "crates/cpl-syntax/", "crates/cpl-next-syntax/", "crates/cpl-diag/",
                       "crates/cpl-prim/", "crates/cpl-config/", "crates/cpl-contract/", "crates/cpl-json/",
                       "crates/cpl-rt/", "Cargo.lock")


WRONG_ACCEPT_INPUTS = ("std/", "crates/cpl-next", "crates/cpl-syntax/", "crates/cpl-prim/", "crates/cpl-diag/",
                       "crates/cpl-config/", "crates/cpl-json/", "crates/cpl-contract/", "spec/conformance/core-05/",
                       "spec/conformance/core-06/", "spec/conformance/std-seq/")


def select(paths, dirs):
    sel = {"all": False, "reason": "", "packages": [], "rust_files": [],
           "script_tests": [], "site": False, "conform_cases": [], "data_filter": "",
           "check_files": [], "audit": False, "idioms_catalog": False, "spec_traits": False, "dreams": False, "wrong_accepts": False, "old_same": False, "slices": [], "steps": []}
    packages, tests, data, steps = set(), set(), [], set()
    step_scripts = ci.step_scripts()
    for path in paths:
        if path.startswith(PACKAGES + "/"):
            sel["audit"] = True
        if path.startswith(("spec/idioms/", "std/", "crates/cpl-check/src/")):
            sel["idioms_catalog"] = True
        if (path.startswith("std/") and path.endswith(".cp")) or path == "spec/conformance/traits.toml":
            sel["spec_traits"] = True
        # An E05xx rename or a std change lands with every dream's migration.
        if path.startswith(("std/", "crates/cpl-check/")):
            sel["dreams"] = True
        # What cpl-next's verdicts read: its crates and the ones they use, std, the guarded suites.
        if path.startswith(WRONG_ACCEPT_INPUTS):
            sel["wrong_accepts"] = True
        crate = next((n for d, n in dirs.items() if path.startswith(d)), None)
        if path.endswith(".rs"):
            sel["rust_files"].append(path)
        editor = next((n for d, n in EDITOR_CRATES.items() if path.startswith(d)), None)
        if matches(path, SITE_SPELLINGS):
            sel["site"] = True
        if path.startswith("spec/conformance/") and path.endswith(".cp"):
            sel["conform_cases"].append(path)
        elif crate:
            packages.add(crate)
        elif editor:
            packages.add(editor)
        elif expr := next((e for p, e in DATA_TESTS if fnmatch.fnmatch(path, p)), None):
            if expr not in data:
                data.append(expr)
            if path.endswith(".cp"):
                sel["check_files"].append(path)
        elif path.endswith(".cp") and matches(path, CHECK_ONLY):
            sel["check_files"].append(path)
        elif matches(path, AUDIT_ONLY):
            continue
        elif path.startswith((".github/workflows/", ".github/actions/")):
            tests.add("scripts/test_workflows.py")
        elif matches(path, PROSE):
            continue
        elif matches(path, EVERY_CRATE):
            packages.update(dirs.values())
        elif matches(path, SITE):
            sel["site"] = True
        elif path in step_scripts:
            steps.add(path)
            if t := script_test(path):
                tests.add(t)
        elif path == "justfile":
            tests.add("scripts/test_justfile.py")
        elif path.startswith("scripts/") and (t := script_test(path)):
            tests.add(t)
        elif path.startswith("scripts/") and ci_only(path):
            continue
        elif path.startswith("handbook/"):
            tests.add("scripts/test_check_handbook.py")
        elif path.startswith(".githooks/"):
            tests.add("scripts/test_check_cache.py")
        elif not sel["all"]:
            # Not `break`: the paths it does map still select their crates, so a land
            # whose hook was cut runs them (277f37f42's spec/core .md hid cpl-check's rdeps).
            sel.update(all=True, reason=path)
    sel["slices"] = [sharded(name, globs, tests, paths) for name, _, globs, tests in SLICES
                     if any(matches(p, slice_globs(globs, tests)) for p in paths)]
    sel["packages"] = sorted(packages)
    sel["script_tests"] = sorted(tests)
    sel["steps"] = sorted(steps)
    sel["data_filter"] = " | ".join(f"({e})" for e in data)
    return sel


PUB_FN_RE = re.compile(r"^\s*pub\s+fn\s+(\w+)\s*[(<\[]")
CALLER_GLOBS = ("spec/conformance/*.cp", f"{PACKAGES}/*/*.cp")


def pub_fn_sigs(text):
    """Each std `pub fn`'s whole signature, by name: from `pub fn` through the line that closes
    its parameter list, so a parameter on its own line is part of it (ea6920634 changed
    matroid_intersection's `weight:` line and flow-matching went red only on main)."""
    sigs, lines, i = {}, text.splitlines(), 0
    while i < len(lines):
        m = PUB_FN_RE.match(lines[i])
        if not m:
            i += 1
            continue
        sig, depth, opened = [], 0, False
        while i < len(lines):
            ln = lines[i]
            sig.append(ln.strip())
            depth += ln.count("(") + ln.count("[") - ln.count(")") - ln.count("]")
            opened = opened or "(" in ln
            i += 1
            if opened and depth <= 0:
                break
        sigs.setdefault(m[1], []).append(" ".join(sig))
    return sigs


def changed_pub_fns(old, new):
    """The `pub fn` names whose signature differs between two `{path: text}` snapshots of std."""
    changed = set()
    for path in set(old) | set(new):
        a, b = pub_fn_sigs(old.get(path, "")), pub_fn_sigs(new.get(path, ""))
        changed |= {n for n in set(a) | set(b) if sorted(a.get(n, [])) != sorted(b.get(n, []))}
    return sorted(changed)


def callers(names):
    """Conformance cases and corpus programs that call one of `names`, with or without type args."""
    if not names:
        return []
    call = re.compile(r"\b(?:" + "|".join(map(re.escape, names)) + r")\s*(?:<[^()\n]*>)?\s*\(")
    out = subprocess.run(["git", "ls-files", "-z", "--", *CALLER_GLOBS], cwd=ROOT,
                         capture_output=True, check=True).stdout
    files = [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]
    return [f for f in files if os.path.isfile(os.path.join(ROOT, f))
            and call.search(Path(ROOT, f).read_text(encoding="utf-8", errors="replace"))]


def importers(paths):
    """Conformance cases and corpus programs that `use` a changed std module: a change inside a
    body, a struct field or a law reaches them with no signature line changing (0c04f356d added
    Action's `replace` law and core-08/laws-are-a-derived-member went red only on main)."""
    mods = sorted({Path(p).stem for p in paths if p.startswith("std/") and p.endswith(".cp")})
    if not mods:
        return []
    use = re.compile(r"^\s*use\s+std\.(?:" + "|".join(map(re.escape, mods)) + r")\b", re.M)
    out = subprocess.run(["git", "ls-files", "-z", "--", *CALLER_GLOBS], cwd=ROOT,
                         capture_output=True, check=True).stdout
    files = [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]
    return [f for f in files if os.path.isfile(os.path.join(ROOT, f))
            and use.search(open(os.path.join(ROOT, f), encoding="utf-8", errors="replace").read())]


def std_cases(*revs):
    """Every case and corpus program a std change between `revs` can break: callers of a changed
    `pub fn` signature, and importers of a changed module."""
    old, new = std_snapshots(*revs)
    return sorted(set(callers(changed_pub_fns(old, new))) | set(importers(new)))


def std_snapshots(*revs):
    """`({path: text}, {path: text})` before and after for the std files `git diff *revs` changes:
    `()` is HEAD against the work tree, `("--cached",)` HEAD against the index, `(a, b)` a against b."""
    out = subprocess.run(["git", "diff", "--name-only", "-z", *revs, "--", "std/*.cp"], cwd=ROOT,
                         capture_output=True, check=True).stdout
    paths = [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]
    before, after = (revs[0], revs[1]) if len(revs) == 2 else ("HEAD", "" if revs else None)

    def read(rev, path):
        if rev is None:
            full = os.path.join(ROOT, path)
            return open(full, encoding="utf-8", errors="replace").read() if os.path.isfile(full) else ""
        r = subprocess.run(["git", "show", f"{rev}:{path}"], cwd=ROOT, capture_output=True)
        return r.stdout.decode("utf-8", "replace") if r.returncode == 0 else ""

    return {p: read(before, p) for p in paths}, {p: read(after, p) for p in paths}


def add_callers(sel, files):
    for f in files:
        key = "conform_cases" if f.startswith("spec/conformance/") else "check_files"
        if f not in sel[key]:
            sel[key].append(f)


def staged():
    out = subprocess.run(["git", "diff", "--cached", "--name-only", "-z"],
                         cwd=ROOT, capture_output=True, check=True).stdout
    return [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]


def changed_since(base):
    out = subprocess.run(["git", "diff", "--name-only", "-z", base, "HEAD"],
                         cwd=ROOT, capture_output=True, check=True).stdout
    return [p.decode("utf-8", "surrogateescape") for p in out.split(b"\0") if p]


def main(argv):
    if argv[:1] == ["--build-slices"]:
        return build_slices(argv[1:])
    if argv[:2] == ["--run-slices", "--base"] and len(argv) > 2:
        return run_slices(argv[3:], argv[2])
    if argv[:1] == ["--run-slices"]:
        return run_slices(argv[1:])
    as_json = "--json" in argv
    argv = [a for a in argv if a != "--json"]
    if argv[:1] == ["--sig-callers"]:
        print("\n".join(std_cases(argv[1], "HEAD")))
        return 0
    if argv[:1] == ["--base"]:
        paths = changed_since(argv[1])
        diff = (argv[1], "HEAD")
    else:
        paths = argv or staged()
        diff = () if argv else ("--cached",)
    sel = select(paths, crate_dirs())
    if any(p.startswith("std/") and p.endswith(".cp") for p in paths) and (argv[:1] == ["--base"] or not argv):
        add_callers(sel, std_cases(*diff))
    if as_json:
        print(json.dumps(sel))
        return 0
    # Shell form for scripts/check.sh: one `key=value` per line.
    print(f"all={int(sel['all'])}")
    print(f"reason={sel['reason']}")
    print("filter=" + " | ".join(f"rdeps({p})" for p in sel["packages"]))
    print("packages=" + " ".join(sel["packages"]))
    print("rust_files=" + " ".join(sel["rust_files"]))
    print("script_tests=" + " ".join(sel["script_tests"]))
    print(f"site={int(sel['site'])}")
    print("conform_cases=" + " ".join(sel["conform_cases"]))
    print("data_filter=" + sel["data_filter"])
    print("check_files=" + " ".join(sel["check_files"]))
    print(f"audit={int(sel['audit'])}")
    print(f"idioms_catalog={int(sel['idioms_catalog'])}")
    print(f"spec_traits={int(sel['spec_traits'])}")
    print(f"dreams={int(sel['dreams'])}")
    print(f"wrong_accepts={int(sel['wrong_accepts'])}")
    print(f"old_same={int(sel['old_same'])}")
    print("slices=" + " ".join(sel["slices"]))
    print("steps=" + " ".join(sel["steps"]))
    print("slice_filter=" + slice_filter(sel["slices"]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
