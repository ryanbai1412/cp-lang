#!/usr/bin/env python3
"""The CI driver's decisions, tested (scripts/test_ci.py): which tests each tier
leaves out, what a CPL_GATE_PART runs, and which checking scripts and spec gates
each tier runs, and every tier after the guards. scripts/check.sh is `ci.py check`:
it sets up the C++ and stage caches, runs the guards, then the tier.

CI splits --fast across parallel jobs (.github/workflows/ci.yml):
  checks            everything but the workspace tests, the conformance ratchet and fmt
  fmt               fmt_check over every .cp file
  conform[:S+S@N/M] the conformance ratchet (only suites S, shard N/M) and, with the
                    conformance suite, the script unit tests
  tests:doc         the doctests
  tests:<partition> that nextest partition (e.g. tests:hash:1/12)
Unset runs all of --fast. With --full, `full` runs only the --full suites (ci-slow.yml).
"""
from __future__ import annotations

import argparse
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

# Which --fast leaves to --full (the reasons are in scripts/check.sh's nextest step).
SLOW_TESTS = (
    "products_and_variable_divisors_are_never_decided_wrongly",
    "front_end_check_is_under_50_ms",
    "recovery_scoreboard",
    "fix_it_scoreboard",
    "committed_scoreboard_ratchet",
    "every_port_compiles_with_every_judges_flags",
)

# Wall-clock assertions (decision 225): a timing budget can fail on a loaded
# machine with no code change, so no blocking tier runs one. CI reports them
# in a job that never fails the workflow (--timing); --full runs them too (ci/R34).
WALL_CLOCK_TESTS = (
    "lsp_requests_are_under_100_ms",
    "keystrokes_on_the_largest_ports_and_a_broken_file_are_within_budget_at_p95",
    "largest_corpus_file_diagnostics_hover_and_completion_p95_under_100ms",
    "the_checks_of_a_typical_function_fit_the_budget",
    "a_segment_tree_monoid_is_checked_within_the_budget",
    "ten_million_steps_fit_the_debugging_budgets",
    "every_command_starts_cold_in_under_100_ms",
)

MODES = ("fast", "commit", "land", "full", "timing")


def _tests(names: tuple[str, ...]) -> str:
    return "".join(f" or test(/(^|::){n}$/)" for n in names)


def slow_filter() -> str:
    """Nextest filter for every test a blocking tier skips."""
    return "binary(budget)" + _tests(SLOW_TESTS) + _tests(WALL_CLOCK_TESTS)


def wall_filter() -> str:
    """Nextest filter for the wall-clock budgets --timing reports."""
    return "binary(budget) or test(/(^|::)front_end_check_is_under_50_ms$/)" + _tests(WALL_CLOCK_TESTS)


def gate_part(part: str, mode: str) -> dict[str, str]:
    """CPL_GATE_PART -> {kind, suites, shard, partition}; raises ValueError on a bad one."""
    out = {"kind": "checks", "suites": "", "shard": "", "partition": ""}
    if part in ("", "checks"):
        return out
    if part == "full" or mode == "full":
        if (part, mode) != ("full", "full"):
            raise ValueError(f"CPL_GATE_PART={part} with --{mode}: only `full` goes with --full")
        out["kind"] = "full"
        return out
    if part in ("fmt", "corpus", "conform", "std-mutants") or part.startswith(("conform:", "tests:")):
        if mode != "fast":
            raise ValueError(f"CPL_GATE_PART={part} needs --fast")
    if part in ("fmt", "corpus", "std-mutants"):
        out["kind"] = part
    elif part == "conform" or part.startswith("conform:"):
        suites = part.partition(":")[2] or "all"
        suites, at, shard = suites.partition("@")
        if at and not shard:
            raise ValueError(f"CPL_GATE_PART={part}: `@` needs a shard N/M")
        out.update(kind="conform", suites=suites.replace("+", " ").replace(",", " "),
                   shard=f"--shard {shard}" if shard else "")
    elif part == "tests:doc":
        out["kind"] = "doc"
    elif part.startswith("tests:") and part != "tests:":
        out.update(kind="tests", partition=part.removeprefix("tests:"))
    else:
        raise ValueError(f"unknown CPL_GATE_PART `{part}` (checks, fmt, corpus, conform, std-mutants, tests:<partition>, or full with --full)")
    return out


ROOT = Path(__file__).resolve().parent.parent
SPEC_GATE = ROOT / "scripts/spec-gate.py"


@dataclass(frozen=True)
class Step:
    """One step check.sh runs through here and what it says when the step fails.
    `argv` starts with a `.py` script's repo path (run with this Python), another
    repo path or a command; `{cpl}` is the debug cpl binary, `{tmp}` a directory
    the tier removes after and `{nproc}` the CPU count. `env` adds variables;
    `needs` names the commands it fails without; `fn(run, cpl)` replaces `argv`.
    A `derived` step checks generated files the regen job rewrites, so it only
    warns once CPL_DERIVED_HEALED=1; a `quiet` one shows its output only on
    failure; `stdout=False` never shows stdout; `drop` hides matching lines."""
    title: str
    argv: tuple[str, ...]
    fail: str
    derived: bool = False
    quiet: bool = False
    stdout: bool = True
    drop: str | None = None
    env: tuple[tuple[str, str], ...] = ()
    needs: tuple[str, ...] = ()
    fn: Callable | None = field(default=None, compare=False)


def gate(*args: str, fail: str, **kw) -> Step:
    """A `scripts/spec-gate.py <gate>` step."""
    return Step(f"spec-gate.py {' '.join(args)}", ("scripts/spec-gate.py", *args), fail, **kw)


def _stale(what: str, name: str) -> str:
    return f"{what} is stale; run: uv run scripts/spec-gate.py {name} --write"


RATCHET = Step("tech-debt ratchet (scripts/debt-ratchet.py --check)", ("scripts/debt-ratchet.py", "--check"),
               "tech-debt ratchet failed; see scripts/debt-ratchet.json")
REFS = Step("every corpus path a file names exists (scripts/corpus.py refs)", ("scripts/corpus.py", "refs"),
            "a file names a corpus package or submission that doesn't exist; the line above names it")
USECASES = Step("use cases: unique IDs, every field, evidence that exists (scripts/check-usecases.py)",
                ("scripts/check-usecases.py", "--check"), "use-case catalogue check failed; see above", stdout=False)

# The board's failing cases generate spec/SUBSET.md's known gaps, so whatever
# rewrites research/scoreboard.json must regenerate it in the same commit.
SUBSET = gate("subset", fail=_stale("spec/SUBSET.md", "subset"))

# Every checking script check.sh runs, by tier, in order: `commit` before a
# commit, `checks` after --fast's build, `full` (with --full's slow suites) last.
STEPS = {
    "commit": (Step("cargo build -p cpl-cli (the cpl the ratchet's corpus_fmt_dirty runs)",
                    ("cargo", "build", "-q", "-p", "cpl-cli"), "cargo build -p cpl-cli failed; see above"),
               RATCHET, REFS, USECASES, SUBSET),
    "checks": (
        RATCHET, REFS,
        Step("handbook reads for an outsider (scripts/check-handbook.py)", ("scripts/check-handbook.py",),
             "handbook doc leans on shared context; see above"),
        USECASES,
        Step("bundled skill (scripts/skill-gen.py --check)", ("scripts/skill-gen.py", "--check", "--cpl", "{cpl}"),
             "bundled skill check failed; rerun: uv run python scripts/skill-gen.py, then with --check", derived=True),
        Step("skill prose uses current spellings (scripts/skill-prose.py)", ("scripts/skill-prose.py", "--cpl", "{cpl}"),
             "inline code in the skill's prose uses a removed spelling; the finding above names the span and its current form"),
        Step("emitted C++ fits 32 KB and every bench program builds (bench/sizes.py --check)",
             ("bench/sizes.py", "--check", "--cpl", "{cpl}"),
             "a program is over the 32 KB bar or a bench program stopped building; see above", drop=": warning: "),
        Step("I/O programs' peak RSS within a fixed chunk of scanf/printf's (bench/rss/run.py --check)",
             ("bench/rss/run.py", "--check", "--cpl", "{cpl}"),
             "an I/O program's peak RSS grows with its input or output; see above"),
        Step("a tiny program starts within 1.02x of C++'s CPU (bench/startup/run.py --check)",
             ("bench/startup/run.py", "--check", "--cpl", "{cpl}"),
             "tiny.cp's startup CPU is over 1.02x tiny.cpp's; see above"),
        Step("deep recursive fns' stack frames within their bounds (bench/frames.py --check)",
             ("bench/frames.py", "--check", "--cpl", "{cpl}"),
             "a recursive fn's g++ -fstack-usage frame is over its bound in bench/frames.py; see above"),
        Step("g++ -O2 compile time of the bench programs within its bars (bench/compile_time.py --check)",
             ("bench/compile_time.py", "--check", "--cpl", "{cpl}"),
             "the bench programs' g++ -O2 time is over a bar in bench/compile_time.py; see above"),
        Step("every std fn with a case grows as its cost: says (bench/scaling.py --check)",
             ("bench/scaling.py", "--check", "--cpl", "{cpl}"),
             "a std fn's time at n and 4n disagrees with its cost:, or research/perf/scaling.json is stale; see above"),
        gate("coverage", fail="a rule lacks the case it needs; see above, then rerun: uv run scripts/spec-gate.py coverage"),
        SUBSET,
        gate("matrix", fail="matrix gate failed; see above"),
        gate("widths", fail=_stale("spec/conformance/widths/", "widths")),
        gate("ownership", fail=_stale("spec/conformance/ownership/", "ownership")),
        gate("xref", fail="a rule id names no rule in spec/core; see above"),
        gate("changelog", fail="a spelling the spec removes has no changelog line; see above"),
        gate("tool-rules", fail="a tool feature in spec/tools/ has no rule id; see above"),
        gate("std-signatures", fail="a spec/lib chapter and std disagree on a return type; see above"),
        gate("decisions-explained", fail="a decision cited in spec/ has no handbook explanation; see above"),
        gate("traits", fail="trait gate failed; see above"),
        gate("sigs", fail="sig gate failed; see above"),
        gate("prelude", fail="prelude gate failed; see above"),
        gate("std-docs", "--audit", fail="a public std fn lacks a required doc label; see above", quiet=True),
        gate("vocab", fail="a spelling spec/GLOSSARY.md bans; see above"),
        gate("markers", fail="a diagnostic case counts lines instead of marking `// <- here`;"
             " rerun: uv run scripts/spec-gate.py markers --fix"),
        Step("std containers print, walk and have is_empty (scripts/std-trait-matrix.py)",
             ("scripts/std-trait-matrix.py", "--max", "0"), "a std container lacks a common trait; see above"),
        Step("std docs follow the std-docs checklist (scripts/std-docs-lint.py)",
             ("scripts/std-docs-lint.py", "--max", "0"), "a std doc breaks a std-docs rule; see above"),
        Step("std signatures follow the std-api checklist (scripts/std-api-lint.py)",
             ("scripts/std-api-lint.py", "--max", "0"), "a std signature breaks a std-api rule; see above"),
        Step("std model cases match their brute force on both engines (scripts/std-models.py)",
             ("scripts/std-models.py",),
             "a std type disagrees with its model; the line above gives the shortest failing op sequence"),
        Step("std's public signatures match their snapshot (scripts/std-api-snapshot.py)",
             ("scripts/std-api-snapshot.py",),
             "std's public API changed; review the diff above and run scripts/std-api-snapshot.py --write"),
        Step("std is lint-clean (scripts/std-lint-ratchet.py)", ("scripts/std-lint-ratchet.py", "--cpl", "{cpl}"),
             "std has a lint hit; fix it in the code"),
        Step("prelude exports what its fns return (scripts/prelude-signatures.py)", ("scripts/prelude-signatures.py",),
             "a prelude fn returns a std type the prelude does not export; see above"),
        Step("std trap examples trap (scripts/std-trap-examples.py)", ("scripts/std-trap-examples.py", "--cpl", "{cpl}"),
             "a std `trap:` doc example did not trap; see above"),
        Step("taste reviews (scripts/review-check.py --check)", ("scripts/review-check.py", "--check"),
             "taste review gate failed; see research/reviews/README.md"),
    ),
    "full": (
        Step("cargo test: the prover fuzzer",
             ("cargo", "test", "-p", "cpl-prover", "--test", "fuzz", "--", "--exact",
              "products_and_variable_divisors_are_never_decided_wrongly"),
             "the prover decided a product or divisor wrong; rerun: cargo test -p cpl-prover --test fuzz"),
        Step("random programs: interpreter, C++ and sanitized C++ match an independent model (conformance/R6)",
             ("scripts/gen-programs.py", "-n", "100", "--seed", "1", "--jobs", "{nproc}", "--out", "{tmp}/gen",
              "--avoid", "rev-pieces", "--cxxflags", "-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG"),
             "a random program's engines disagree with the model; see above, then rerun: uv run scripts/gen-programs.py -n 100 --seed 1"),
        Step("programs derived from the grammar (scripts/grammar-fuzz.py)",
             ("scripts/grammar-fuzz.py", "--n", "300", "--check", "--max-syntax", "231", "--show", "0"),
             "a program derived from the grammar crashed cpl, or more derived programs are syntax errors; see above"),
        gate("metamorphic", fail="a meaning-preserving edit changed a case's output; see above"),
        gate("speed", fail="the conformance and spec-block suites went over their CPU budget; see above"),
        gate("explain-copies", "--max", "0",
             fail="cpl explain claims a copy the C++ doesn't make; see above"),
        gate("tool-vocab", fail="help, explain, an error, a std doc or a hover uses a spelling spec/GLOSSARY.md bans; see above"),
        gate("flakes", "--runs", "2", fail="a conformance case changed verdict between runs; see above"),
        gate("case-bugs", fail="a known-gap case fails because of the case itself (conformance/R3); fix the case"
             " named above, then rerun: uv run scripts/spec-gate.py case-bugs"),
        Step("cargo test: the committed scoreboard ratchet",
             ("cargo", "test", "-p", "cpl-conform", "--test", "ratchet", "--", "--exact",
              "committed_scoreboard_ratchet"),
             "a passing case regressed against research/scoreboard.json; rerun: cargo test -p cpl-conform --test ratchet"),
        Step("cargo test: every port compiled with every judge profile's flags",
             ("cargo", "test", "-p", "cpl-cli", "--test", "judges", "--", "--exact",
              "every_port_compiles_with_every_judges_flags"),
             "a port fails to compile under a judge profile; rerun: cargo test -p cpl-cli --test judges",
             needs=("g++",)),
        Step("cargo test: the budget tests, alone", ("cargo", "test", "-p", "cpl-cli", "--test", "budget", "--",
                                                     "--test-threads=1"),
             "a budget test is over; rerun: cargo test -p cpl-cli --test budget -- --test-threads=1"),
        Step("cargo test: the front-end latency, alone",
             ("cargo", "test", "-p", "cpl-lsp", "--test", "latency", "--", "--exact", "front_end_check_is_under_50_ms"),
             "the front-end check is over 50 ms; rerun: cargo test -p cpl-lsp --test latency"),
        Step("wasm build and crates/cpl-wasm/tests/smoke.mjs", ("scripts/build-wasm.sh", "--smoke", "--out", "{tmp}/wasm"),
             "the wasm build or its smoke test failed; rerun: scripts/build-wasm.sh --smoke", needs=("node",)),
        Step("scripts/site-check.sh: site build, cpl.wasm size cap, browser smoke", ("scripts/site-check.sh", "--no-cache"),
             "the site build, its cpl.wasm cap or its browser smoke failed; rerun: CPL=target/debug/cpl scripts/site-check.sh",
             env=(("CPL", "{cpl}"),), needs=("node",)),
        Step("editors/vscode: npm test", (), "the VS Code extension tests failed; rerun: cd editors/vscode && npm test",
             needs=("node", "npm"), fn=lambda run, cpl: vscode_tests(run)),
        Step("editors/smoke.sh: the other editor setups against cpl lsp", ("editors/smoke.sh",),
             "an editor setup failed against cpl lsp; rerun: editors/smoke.sh"),
    ),
}


def vscode_tests(run=subprocess.run) -> int:
    """The extension's unit tests, then its extension-host suite under a display or xvfb-run."""
    ext = ROOT / "editors/vscode"
    cmds = [(["cargo", "build", "-q", "-p", "cpl-cli"], ROOT)]  # the LSP test starts target/debug/cpl
    if not (ext / "node_modules").is_dir():
        cmds.append((["npm", "ci", "--no-audit", "--no-fund"], ext))
    cmds.append((["npm", "test"], ext))
    host = ["npm", "run", "test:integration"]
    if os.environ.get("DISPLAY"):
        cmds.append((host, ext))
    elif shutil.which("xvfb-run"):
        cmds.append((["xvfb-run", "-a", *host], ext))
    else:
        host = None
    for cmd, cwd in cmds:
        if run(cmd, cwd=cwd).returncode:
            return 1
    if host is None:
        print("editors/vscode: SKIPPED extension-host tests (no display and no xvfb-run)")
    return 0


# The tiers a touched checking script's own step comes from; the full tier's are too slow.
TOUCHED_TIERS = ("commit", "checks")


def touched_steps(scripts) -> list[Step]:
    """The commit and checks tier steps that run one of `scripts`, each once, in tier order."""
    out = []
    for tier in TOUCHED_TIERS:
        out += [s for s in STEPS[tier] if s.argv[0] in scripts and s not in out]
    return out


def step_scripts() -> set[str]:
    """Every script a commit or checks tier step runs."""
    return {s.argv[0] for tier in TOUCHED_TIERS for s in STEPS[tier]}


def _argv(step: Step, values: dict[str, str]) -> list[str]:
    """The command `step` runs, its placeholders filled in."""
    first, *args = (a.format(**values) for a in step.argv)
    if first.endswith(".py"):
        return [sys.executable, str(ROOT / first), *args]
    return [str(ROOT / first) if "/" in first else first, *args]


def run_steps(tier: str, cpl: str = "target/debug/cpl", run=subprocess.run, healed: bool | None = None,
              only=None, keep_going: bool = False) -> int:
    """Runs the tier's steps, or with `only` the touched steps running those scripts, in order,
    and stops at the first that fails; `keep_going` runs them all and names every failure."""
    if healed is None:
        healed = os.environ.get("CPL_DERIVED_HEALED") == "1"
    failed: list[str] = []
    with tempfile.TemporaryDirectory(prefix="cpl-ci-") as tmp:
        values = {"cpl": cpl, "tmp": tmp, "nproc": str(os.cpu_count() or 1)}
        for step in touched_steps(only) if only else STEPS[tier]:
            print(f"\n==> {step.title}", flush=True)
            missing = [c for c in step.needs if not shutil.which(c)]
            if missing:
                print(f"check: `{missing[0]}` is not installed; {step.title} needs it", file=sys.stderr)
                if not keep_going:
                    return 1
                failed.append(step.title)
                continue
            kw = {"cwd": ROOT, "text": True}
            if step.env:
                kw["env"] = {**os.environ, **{k: v.format(**values) for k, v in step.env}}
            if step.fn:
                done = subprocess.CompletedProcess((), step.fn(run, cpl))
            elif step.drop:
                done = run(_argv(step, values), stdout=subprocess.PIPE, stderr=subprocess.STDOUT, **kw)
                sys.stdout.writelines(l for l in done.stdout.splitlines(True) if step.drop not in l)
            elif step.quiet:
                done = run(_argv(step, values), capture_output=True, **kw)
                if done.returncode:
                    sys.stdout.write(done.stdout)
                    sys.stderr.write(done.stderr)
            else:
                done = run(_argv(step, values), stdout=None if step.stdout else subprocess.DEVNULL, **kw)
            sys.stdout.flush()
            if done.returncode == 0:
                continue
            if step.derived and healed:
                print(f"warning: {step.fail} (the regen job commits the fix)", file=sys.stderr)
                continue
            print(f"check: {step.fail}", file=sys.stderr)
            if not keep_going:
                return 1
            failed.append(step.title)
    for title in failed:
        print(f"check: failed: {title}", file=sys.stderr)
    return 1 if failed else 0


# The --fast tier after its build: the workspace tests and what overlaps them. Each
# backgrounded job writes a log that is shown when its step comes round.
CHECK_CACHE = ROOT / "scripts/check_cache.py"
SCRIPT_TESTS = ("scripts/test_*.py", "bench/test_*.py", "tools/*/test_*.py")
TRACE_TESTS = ("tools/trace/trace.test.cjs", "tools/trace/state.test.cjs")
# Besides SLOW_TESTS, what the fallback `cargo test` (no nextest) leaves out.
UNTIMED_TESTS = ("run_answers_within_200_ms", "build_takes_under_a_second_and_fits_64_kb")
CACHED = "cached (tree unchanged outside its excludes)"
GIT_SCOPE = ("GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE", "GIT_OBJECT_DIRECTORY", "GIT_PREFIX")


def target_dir() -> Path:
    """Cargo's target directory, as scripts/gate-env.sh resolves it."""
    t = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
    return t if t.is_absolute() else ROOT / t


def bare_home(home: str, env=None) -> dict[str, str]:
    """A CI runner's environment for the script tests: an empty HOME, no system git config
    and no git identity, while the toolchains keep their real homes. A commit hook's
    GIT_DIR and GIT_INDEX_FILE are dropped, or a test writes its scratch repo into the
    real index."""
    env = dict(os.environ if env is None else env)
    real = env.get("HOME", "")
    keep = {"CARGO_HOME": f"{real}/.cargo", "RUSTUP_HOME": f"{real}/.rustup",
            "MISE_DATA_DIR": f"{real}/.local/share/mise", "MISE_CONFIG_DIR": f"{real}/.config/mise",
            "UV_CACHE_DIR": f"{real}/.cache/uv"}
    out = {k: v for k, v in env.items() if k not in GIT_SCOPE}
    out.update({k: env.get(k, v) for k, v in keep.items()})
    out.update(HOME=home, XDG_CONFIG_HOME=f"{home}/.config", GIT_CONFIG_NOSYSTEM="1")
    return out


def prelude_names(text: str) -> int:
    """The names std/prelude.cp's `{...}` groups export."""
    flat = "".join(line.split("//")[0] for line in text.splitlines())
    groups = re.findall(r"\{[^}]*\}", flat)
    return sum(1 for g in groups for name in g.split(",") if re.search(r"[A-Za-z_]", name))


def open_traps(text: str) -> int:
    """The Trap Parity Gate's open register in crates/cpl-cli/tests/known_traps.cp."""
    return sum(1 for line in text.splitlines() if line.startswith("if which == "))


@dataclass
class Tail:
    """check.sh's --fast tier from its build on (`ci.py tail`). `manifest` empty turns the
    stage cache off; `sel_*` are the land train's touched-path selection."""
    mode: str
    gate_part: str = ""
    cpl: str = "target/debug/cpl"
    cache_dir: str = ""
    manifest: str = ""
    sel_packages: str = ""
    sel_filter: str = ""
    sel_site: str = ""
    sel_slices: str = ""
    sel_slice_filter: str = ""
    only: frozenset[str] | None = None
    progress: str = ""
    run: Callable = subprocess.run
    popen: Callable = subprocess.Popen
    which: Callable = shutil.which
    clock: Callable = time.monotonic
    times: list[tuple[int, str]] = field(default_factory=list)
    _jobs: dict = field(default_factory=dict)
    _step: tuple[str, float] | None = None

    def step(self, title: str) -> None:
        self.end_step()
        self._step = (title, self.clock())
        print(f"\n==> {title}", flush=True)

    def wants(self, key: str) -> bool:
        """Whether the commit tier runs step `key` (every step, or those `--only`
        names); from here on, `progress` lists it and the steps after it as unrun."""
        if self.progress:
            Path(self.progress).write_text("\n".join(COMMIT_STEPS[COMMIT_STEPS.index(key):]) + "\n")
        return self.only is None or key in self.only

    def end_step(self) -> None:
        if self._step:
            self.times.append((int(self.clock() - self._step[1]), self._step[0]))
            self._step = None

    def _cache(self, verb: str, stage: str) -> bool:
        if not self.manifest:
            return verb == "save"
        done = self.run([sys.executable, str(CHECK_CACHE), verb, self.cache_dir, stage, self.manifest], cwd=ROOT)
        return done.returncode == 0

    def hit(self, stage: str) -> bool:
        return self._cache("check", stage)

    def save(self, stage: str) -> None:
        self._cache("save", stage)

    def need(self, cmd: str, why: str) -> None:
        if not self.which(cmd):
            raise Failed(f"`{cmd}` is not installed; {why}")

    def call(self, argv: list[str], **kw) -> int:
        return self.run(argv, cwd=ROOT, **kw).returncode

    def spawn(self, name: str, argv: list[str], logs: str, env=None) -> None:
        log = open(Path(logs, f"{name}.log"), "w+")
        self._jobs[name] = (self.popen(argv, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, env=env), log)

    def started(self, name: str) -> bool:
        return name in self._jobs

    def join(self, name: str) -> int:
        """Waits for a backgrounded job, shows its log and returns its exit code."""
        proc, log = self._jobs.pop(name)
        code = proc.wait()
        log.seek(0)
        sys.stdout.write(log.read())
        sys.stdout.flush()
        log.close()
        return code

    def stop_all(self) -> None:
        for proc, log in self._jobs.values():
            if proc.poll() is None:
                proc.kill()
                proc.wait()
            log.close()
        self._jobs.clear()

    def __call__(self) -> int:
        return self._guarded(self._tier)

    def commit(self, sel: dict[str, str]) -> int:
        """The commit tier: the checks scripts/affected.py's selection `sel` reaches."""
        return self._guarded(lambda logs: self._commit(sel, logs))

    def _guarded(self, body: Callable[[str], None]) -> int:
        # A killed tier stops its backgrounded jobs too: SIGTERM unwinds like Ctrl-C.
        signal.signal(signal.SIGTERM, _interrupt)
        with tempfile.TemporaryDirectory(prefix="cpl-tail-") as logs:
            try:
                body(logs)
            except Failed as e:
                if str(e):
                    print(f"check: {e}", file=sys.stderr)
                return 1
            finally:
                self.stop_all()
                self.end_step()
        return 0

    def wasm(self) -> bool:
        """The playground build. CI's site job builds cpl-wasm itself, and a lane box
        without the wasm32 target warns and skips. Whether it checked."""
        self.step("cargo check -p cpl-wasm --target wasm32-unknown-unknown (the playground build)")
        if os.environ.get("CI"):
            print("skipped in CI: the site build job builds cpl-wasm")
            return False
        lib = self.run(["rustc", "--print", "target-libdir", "--target", "wasm32-unknown-unknown"],
                       cwd=ROOT, capture_output=True, text=True)
        libdir = lib.stdout.strip() if lib.returncode == 0 else ""
        if not libdir or not Path(libdir).is_dir():
            print("WARNING: skipped; the wasm32 target is not installed (run: rustup target add wasm32-unknown-unknown)",
                  file=sys.stderr)
            return False
        env = {**os.environ, "RUSTFLAGS": f"{os.environ.get('RUSTFLAGS', '')} -D warnings"}
        if self.call(["cargo", "check", "-q", "-p", "cpl-wasm", "--target", "wasm32-unknown-unknown"], env=env):
            raise Failed("cpl-wasm does not check for wasm32; rerun:"
                         " RUSTFLAGS=-Dwarnings cargo check -p cpl-wasm --target wasm32-unknown-unknown")
        return True

    def wasm_data(self, pkgs: list[str]) -> None:
        """The data segment of a names build of cpl.wasm (std's outlines and every
        crate's strings) and each touched crate within its budget, so a change that
        grows one fails at commit and land time, not later in the site job."""
        self.step("cpl.wasm data segment and touched crates within their budgets (scripts/build-wasm.sh --names)")
        out = target_dir() / "wasm-data-gate"
        if self.call(["scripts/build-wasm.sh", "--names", str(out)]):
            raise Failed(f"scripts/build-wasm.sh --names failed; rerun: scripts/build-wasm.sh --names {out}")
        crates = [p.replace("-", "_") for p in pkgs]
        # Held to the parent's sizes like land.sh's check: a crate main already has over
        # its budget refuses only if it grew, so a lane can still cut it back under.
        base = self.run(["git", "merge-base", "HEAD", "origin/main"], cwd=ROOT, capture_output=True, text=True)
        parent = self.run(["scripts/wasm-warm.sh", "--parent", base.stdout.strip() or "HEAD", *crates],
                          cwd=ROOT, capture_output=True, text=True)
        held = ["--parent", parent.stdout.strip()] if parent.returncode == 0 and parent.stdout.strip() else []
        if not held:
            print("note: no wasm sizes of the parent (scripts/wasm-warm.sh); touched crates over budget only"
                  " warn here, land.sh and CI's per-tip ratchet hold them to the parent")
        argv = ["node", "web/scripts/wasm-crates.mjs", "--data", str(out / "cpl.wasm")]
        if held:
            argv += [*held, "--touched", ",".join(crates)]
        if self.call(argv):
            raise Failed("cpl.wasm's data segment or a touched crate is over its budget (named above); rerun: "
                         + " ".join(argv))

    def _build(self) -> None:
        self.step("cargo fmt --all --check")
        if self.call(["cargo", "fmt", "--all", "--check"]):
            raise Failed("Rust files are not formatted; run: cargo fmt --all")
        self.step("cargo clippy --workspace --all-targets -- -D warnings")
        if self.hit("clippy"):
            print(CACHED)
        elif self.call(["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]):
            raise Failed("clippy found warnings; rerun: cargo clippy --workspace --all-targets -- -D warnings")
        else:
            self.save("clippy")
        self.wasm()
        self.step("cargo build -p cpl-cli -p cpl-conform (the binaries the checks below run)")
        if self.call(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"]):
            raise Failed("cargo build -p cpl-cli -p cpl-conform failed")

    def _commit(self, sel: dict[str, str], logs: str) -> None:
        # First: the hook's 25 s cut allows a commit past any other step, so a slice
        # after them went unrun whenever they were slow (e843b83dc's unparsable std).
        if self.only is None:
            self._slices(sel.get("slices", "").split())
        if self.wants("scripts"):
            self.step("checking scripts (scripts/ci.py steps commit)")
            if run_steps("commit", self.cpl, run=self.run):
                raise Failed("")
        if self.only is None:
            self.step("generated files current: skill pages, gap lists (scripts/gen.py --check)")
            # CI's regen job regenerates stale files on main, so a commit isn't blocked by
            # staleness it may not have caused; only the summary line, the rest is the job's.
            gen = self.run([sys.executable, "scripts/gen.py", "--check"], cwd=ROOT, stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT, text=True)
            if gen.returncode:
                print((gen.stdout.strip().splitlines() or [""])[-1].removesuffix("; run: just gen"))
                print("note: not blocking, and no need to regenerate them: CI's regen job rewrites these on main")
        not_slow = f" & not ({slow_filter()})"
        pkgs = sel.get("packages", "").split()
        ps = [a for p in pkgs for a in ("-p", p)]
        if pkgs and self.wants("fmt"):
            self.step("cargo fmt --check of the touched crates")
            if self.call(["cargo", "fmt", "--check", *ps]):
                raise Failed(f"Rust files are not formatted; run: cargo fmt {' '.join(ps)}")
        if pkgs and self.wants("clippy"):
            self.step("cargo clippy of the touched crates and nextest of the crates they reach")
            if self.call(["cargo", "clippy", "-q", *ps, "--all-targets", "--", "-D", "warnings"]):
                raise Failed(f"clippy found warnings; rerun: cargo clippy {' '.join(ps)} --all-targets -- -D warnings")
        if pkgs and self.wants("wasm"):
            tree = self.run(["cargo", "tree", "-q", "-p", "cpl-wasm", "--target", "wasm32-unknown-unknown", "-e",
                             "normal", "--prefix", "none"], cwd=ROOT, capture_output=True, text=True)
            if tree.returncode:
                raise Failed(f"cargo tree -p cpl-wasm failed:\n{tree.stderr}")
            if {line.split()[0] for line in tree.stdout.splitlines() if line.strip()} & set(pkgs) and self.wasm():
                self.wasm_data(pkgs)
        if pkgs and self.wants("nextest"):
            expr = f"({sel.get('filter', '')}){not_slow}"
            if slice_filter := sel.get("slice_filter"):
                expr += f" & not ({slice_filter})"
            if self.call(["cargo", "nextest", "run", "--workspace", "--profile", "commit", "--no-fail-fast", "-E", expr]):
                raise Failed(f"the touched crates' tests failed (named above); rerun: cargo nextest run --workspace -E '{expr}'")
        if (data := sel.get("data_filter")) and self.wants("data"):
            self.step(f"tests that read the touched data files: {data}")
            expr = f"({data}){not_slow}"
            if self.call(["cargo", "nextest", "run", "--workspace", "--no-fail-fast", "-E", expr]):
                raise Failed(f"tests that read the touched data files failed (named above); rerun: cargo nextest run --workspace -E '{expr}'")
        if (files := sel.get("check_files", "").split()) and self.wants("check_files"):
            self.step("cpl check of the touched corpus programs")
            self._cargo_build("cpl-cli")
            for f in files:
                if (ROOT / f).is_file() and self.call([self.cpl, "check", f]):
                    raise Failed(f"{f} has check errors; cpl check's message above says how to fix it")
        if sel.get("audit") == "1" and self.wants("audit"):
            self.step("staged corpus packages pass every mode (scripts/corpus.py verify --changed staged)")
            self._cargo_build("cpl-cli")
            if self.call([sys.executable, "scripts/corpus.py", "make", "--changed", "staged", "--cpl", self.cpl], stdout=subprocess.DEVNULL):
                raise Failed("scripts/corpus.py make --changed staged failed; rerun it to see which package")
            if self.call([sys.executable, "scripts/corpus.py", "verify", "--changed", "staged", "--skip", "dream,header", "--cpl", self.cpl]):
                raise Failed("a staged corpus package fails scripts/corpus.py verify; the finding above names the mode and the package")
            staged = self.run(["git", "diff", "--cached", "--name-only", "--diff-filter=AMR", "--", "corpus/problems/*/dream.cp"], cwd=ROOT, capture_output=True, text=True)
            if dreams := [Path(f).parent.name for f in staged.stdout.split()]:
                self.step("staged dreams agree with the accepted C++ (scripts/corpus.py verify --only dream)")
                if self.call([sys.executable, "scripts/corpus.py", "verify", "--only", "dream", "--cpl", self.cpl, *dreams]):
                    raise Failed("a staged dream.cp disagrees with its accepted C++ or meta.json lacks its record;"
                                 " run scripts/corpus.py record <id> and stage meta.json")
        if sel.get("idioms_catalog") == "1" and self.wants("idioms"):
            self.step("lint catalog names only real APIs (scripts/idioms-catalog.py --check)")
            if self.call([sys.executable, "scripts/idioms-catalog.py", "--check"]):
                raise Failed("a spec/idioms/lints.md row names an API std doesn't define; the finding above names the lint and the name")
        if sel.get("spec_traits") == "1" and self.wants("traits"):
            self.step("every std trait impl has its traits.toml case (scripts/spec-gate.py traits)")
            if self.call([sys.executable, "scripts/spec-gate.py", "traits"]):
                raise Failed("a std impl's trait method has no spec/conformance/traits.toml case; write the case it names"
                             " (or regenerate: scripts/spec-gate.py traits --write)")
        if (steps := sel.get("steps", "").split()) and self.wants("touched_steps"):
            self.step(f"touched checking scripts (scripts/ci.py steps touched {' '.join(steps)})")
            if run_steps("touched", self.cpl, run=self.run, only=set(steps)):
                raise Failed("")
        if (tests := sel.get("script_tests", "").split()) and self.wants("script_tests"):
            self.step(f"script unit tests: {' '.join(tests)}")
            home = tempfile.mkdtemp(prefix="cpl-home-", dir=logs)
            if self.call([sys.executable, "-m", "unittest", *tests], env=bare_home(home)):
                raise Failed(f"script unit tests failed (named above); rerun: uv run python -m unittest {' '.join(tests)}")
        if (cases := sel.get("conform_cases", "").split()) and self.only is None:
            self.step("staged conformance cases (scripts/conform_staged.py)")
            if self.call(["git", "diff", "--quiet", "--", "crates", "std", "Cargo.toml", "Cargo.lock"]):
                raise Failed("unstaged changes under crates/ or std/ would run the staged cases against code this commit"
                             " leaves out (4aaa8b0db); stage them, or stash them with git stash --keep-index")
            self._cargo_build("cpl-cli", "cpl-conform")
            if self.call([sys.executable, "scripts/conform_staged.py", str(target_dir() / "debug/cpl-conform"), *cases]):
                raise Failed("a staged conformance case fails; cpl-conform's message above says how to fix it")
        if sel.get("site") == "1" and self.wants("site"):
            self.step("site snippet gate (node web/scripts/gate.mjs)")
            if self.call(["node", "web/scripts/gate.mjs"]):
                raise Failed("site snippet gate failed; rerun: node web/scripts/gate.mjs")

    def _cargo_build(self, *pkgs: str) -> None:
        if self.call(["cargo", "build", "-q", *(a for p in pkgs for a in ("-p", p))]):
            raise Failed(f"cargo build {' '.join('-p ' + p for p in pkgs)} failed")

    def _tier(self, logs: str) -> None:
        self._build()
        checks = self.gate_part == "checks"
        cpl_env = {**os.environ, "CPL": self.cpl}
        gate_rerun = f"site snippet gate failed; rerun: CPL={self.cpl} node web/scripts/gate.mjs"
        # CI's checks part runs no tests, so the snippet gate overlaps fmt_check instead.
        if checks and not self.hit("site_gate"):
            self.need("node", "the site's snippet gate runs on it")
            self.spawn("site_gate", ["node", "web/scripts/gate.mjs"], logs, env=cpl_env)
        fmt = "skip" if checks else "cached" if self.hit("fmt_corpus") else "run"
        if fmt == "run":
            self.spawn("fmt_corpus", [sys.executable, "scripts/fmt_check.py", "--cpl", self.cpl], logs)
        board = "skip" if checks else "cached" if self.hit("scoreboard_fast") else "run"
        home = tempfile.mkdtemp(prefix="cpl-home-", dir=logs)
        if board == "run":
            tests = self.run(["git", "ls-files", *SCRIPT_TESTS], cwd=ROOT, capture_output=True,
                             text=True).stdout.split()
            self.spawn("script_tests", [sys.executable, "-m", "unittest", *tests], logs, env=bare_home(home))
            self.spawn("scoreboard", [str(Path(self.cpl).parent / "cpl-conform"), "all", "--check"], logs)
        self._tests()
        # The recovery scoreboard checks thousands of broken ports on every core, so it
        # starts after the tests and overlaps the checks below.
        recovery = "skip" if checks else "cached" if self.hit("recovery") else "run"
        if recovery == "run":
            self.spawn("recovery", ["cargo", "test", "-q", "-p", "cpl-cli", "--test", "recovery"], logs)

        self._corpus_static(checks)
        self.step("std/prelude.cp: name count")
        print(f"prelude: {prelude_names((ROOT / 'std/prelude.cp').read_text(encoding='utf-8'))} names")

        self.step("cpl fmt is idempotent and keeps meaning on every .cp file")
        if fmt == "skip":
            print("skipped: the CI fmt part runs it (CPL_GATE_PART=checks)")
        elif fmt == "cached":
            print(CACHED)
        else:
            if self.join("fmt_corpus"):
                raise Failed("cpl fmt is unstable or changes meaning; rerun: scripts/fmt_check.py")
            self.save("fmt_corpus")

        self.step("checking scripts (scripts/ci.py steps checks)")
        if run_steps("checks", self.cpl, run=self.run):
            raise Failed("a checking script failed; the step above names it")

        self.step("site snippet gate (node web/scripts/gate.mjs)")
        if self.started("site_gate"):
            if self.join("site_gate"):
                raise Failed(gate_rerun)
            self.save("site_gate")
        elif self.hit("site_gate"):
            print(CACHED)
        elif self.mode == "land" and self.sel_site != "1" and not self.sel_packages:
            print("skipped: the train touches no site, docs or crate path")
        else:
            self.need("node", "the site's snippet gate runs on it")
            if self.call(["node", "web/scripts/gate.mjs"], env=cpl_env):
                raise Failed(gate_rerun)
            self.save("site_gate")

        self.step("trace viewer tests (node tools/trace/*.test.cjs)")
        self.need("node", "the trace viewer's tests run on it")
        for t in TRACE_TESTS:
            if self.call(["node", "--test-reporter=dot", t]):
                raise Failed(f"trace viewer tests failed; rerun: node {t}")

        self.step("cpl-conform ratchet (all --check against research/scoreboard.json)")
        if board == "skip":
            print("skipped: the CI conform part runs it and the script tests (CPL_GATE_PART=checks)")
        elif board == "cached":
            print(CACHED)
        else:
            scripts, ratchet = self.join("script_tests"), self.join("scoreboard")
            if scripts:
                raise Failed("script unit test failure; rerun: uv run python -m unittest scripts/test_*.py")
            if ratchet:
                raise Failed("cpl-conform ratchet failure; rerun: cargo run -p cpl-conform -- all --check")
            self.save("scoreboard_fast")

        self.step("cargo test: the recovery scoreboard" + (" (backgrounded)" if recovery == "run" else ""))
        if recovery == "skip":
            print("skipped: the CI test partitions run it (CPL_GATE_PART=checks)")
        elif recovery == "cached":
            print(CACHED)
        else:
            if self.join("recovery"):
                raise Failed("recovery scoreboard failed; rerun: cargo test -p cpl-cli --test recovery")
            self.save("recovery")

        if self.mode == "full":
            self.step("the --full suites (scripts/ci.py steps full)")
            if run_steps("full", self.cpl, run=self.run):
                raise Failed("a --full suite failed; the step above names it")
        self.end_step()
        traps = open_traps((ROOT / "crates/cpl-cli/tests/known_traps.cp").read_text(encoding="utf-8"))
        print(f"\nknown_traps: {traps} open (crates/cpl-cli/tests/known_traps.cp)")

    def _corpus_static(self, checks: bool = False) -> None:
        self.step("every corpus package passes the static modes (scripts/corpus.py verify --no-run)")
        if checks:
            print("skipped: the CI corpus part runs it (CPL_GATE_PART=checks)")
            return
        if self.call([sys.executable, "scripts/corpus.py", "verify", "--no-run", "--only",
                      "header,check,package,lint", "--cpl", self.cpl]):
            raise Failed("a corpus package fails scripts/corpus.py verify; the finding above names the mode and the package")

    def part(self, since: float) -> int:
        """A CI gate part other than checks (CPL_GATE_PART=conform..., tests:..., fmt, corpus);
        `since` is when the gate started, for its closing line."""
        try:
            done = self._part(gate_part(self.gate_part, self.mode))
        except (Failed, ValueError) as e:
            print(f"check: {e}", file=sys.stderr)
            return 1
        finally:
            self.end_step()
        print(f"\ncheck: {done} passed in {int(time.time() - since)}s")
        return 0

    def _part(self, g: dict[str, str]) -> str:
        env = os.environ
        if g["kind"] == "conform":
            suites = g["suites"].split()
            if {"all", "conformance"} & set(suites):
                self.step("script unit tests")
                with tempfile.TemporaryDirectory(prefix="cpl-home-") as home:
                    tests = self.run(["git", "ls-files", *SCRIPT_TESTS], cwd=ROOT, capture_output=True,
                                     text=True).stdout.split()
                    if self.call([sys.executable, "-m", "unittest", *tests], env=bare_home(home)):
                        raise Failed("script unit test failure; rerun: uv run python -m unittest scripts/test_*.py")
            self.step(f"cpl-conform ratchet ({g['suites']} --check against research/scoreboard.json)")
            if self.call(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"]):
                raise Failed("cargo build -p cpl-cli -p cpl-conform failed")
            # Seeded problems (problem-packages/R9) read their cases from
            # target/corpus-tests/; only the corpus suite reads them.
            if {"all", "corpus"} & set(suites):
                if self.call([sys.executable, "scripts/corpus.py", "make", "--cpl", self.cpl], stdout=subprocess.DEVNULL):
                    raise Failed("scripts/corpus.py make failed; rerun it to see which package")
            # CI keeps main's last green run in $CPL_CONFORM_PREV, so a case newer than
            # research/scoreboard.json still can't regress unseen (ci.yml).
            prev = env.get("CPL_CONFORM_PREV")
            argv = [str(target_dir() / "debug/cpl-conform"), *suites, *g["shard"].split(), "--check",
                    *(["--prev", prev, "--save", f"{prev}.new"] if prev else [])]
            if self.call(argv):
                raise Failed(f"cpl-conform ratchet failure; rerun: cargo run -p cpl-conform -- "
                             f"{g['suites']} {g['shard']} --check")
            return "conformance part"
        if g["kind"] == "doc":
            self.step("cargo test --workspace --doc")
            if self.call(["cargo", "test", "-q", "--workspace", "--doc"]):
                raise Failed("a doctest failed; rerun: cargo test --workspace --doc")
            return "doctests"
        if g["kind"] == "tests":
            # CI compiles the test binaries once (cargo nextest archive) and every
            # partition runs them from CPL_NEXTEST_ARCHIVE; unset, nextest builds.
            archive = env.get("CPL_NEXTEST_ARCHIVE")
            src = (["--archive-file", archive, "--workspace-remap", ".", "--extract-to", ".", "--extract-overwrite"]
                   if archive else ["--workspace"])
            self.step(f"cargo nextest run --partition {g['partition']} (without the --full tests)")
            # No retries: a flaky test fails on its first run (ci/R35).
            if self.call(["cargo", "nextest", "run", *src, "--profile", "gate", "--no-fail-fast",
                          "--partition", g["partition"], "-E", f"not ({slow_filter()})"]):
                raise Failed("a test failed; nextest's summary above names it, rerun: cargo nextest run -E 'test(<name>)'")
            return f"test partition {g['partition']}"
        if g["kind"] == "fmt":
            self.step("cpl fmt is idempotent and keeps meaning on every .cp file")
            if self.hit("fmt_corpus"):
                print(CACHED)
            else:
                if self.call(["cargo", "build", "-q", "-p", "cpl-cli"]):
                    raise Failed("cargo build -p cpl-cli failed")
                if self.call([sys.executable, "scripts/fmt_check.py", "--cpl", self.cpl]):
                    raise Failed("cpl fmt is unstable or changes meaning; rerun: scripts/fmt_check.py")
                self.save("fmt_corpus")
            return "fmt part"
        if g["kind"] == "corpus":
            if self.call(["cargo", "build", "-q", "-p", "cpl-cli"]):
                raise Failed("cargo build -p cpl-cli failed")
            self._corpus_static()
            return "corpus part"
        if g["kind"] == "std-mutants":
            return self._std_mutants(env)
        if g["kind"] == "full":
            self.step("cargo build -p cpl-cli -p cpl-conform (the binaries the --full suites run)")
            if self.call(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"]):
                raise Failed("cargo build -p cpl-cli -p cpl-conform failed")
            self.step("the --full suites (scripts/ci.py steps full)")
            if run_steps("full", self.cpl, run=self.run, keep_going=True):
                raise Failed("a --full suite failed; the lines above name each one")
            return "full part"
        raise ValueError(f"CPL_GATE_PART={self.gate_part} runs through `ci.py tail`")

    def _std_mutants(self, env) -> str:
        """Every mutant of the std/*.cp lines this push changed is caught by std's own
        suites (stdlib.md "Tests catch real bugs"); the base is CPL_STD_MUTANTS_BASE,
        else CHECK_BASE, else origin/main."""
        base = env.get("CPL_STD_MUTANTS_BASE") or env.get("CHECK_BASE") or "origin/main"
        self.step(f"std mutants since {base} (scripts/mutation-score.py --std --since {base} --min 1.0)")
        # CI restores main's `ci`-profile target/ (setup-rust target: release).
        profile = "ci" if env.get("CI") else "release"
        if self.call(["cargo", "build", "-q", "--profile", profile, "-p", "cpl-cli", "-p", "cpl-conform"]):
            raise Failed(f"cargo build --profile {profile} -p cpl-cli -p cpl-conform failed")
        argv = [sys.executable, "scripts/mutation-score.py", "--std", "--since", base, "--min", "1.0",
                "--bin", str(target_dir() / profile), "-j", str(os.cpu_count() or 2)]
        if self.call(argv):
            raise Failed(f"a std mutant survives; add a test that kills each `missed:` line above, rerun:"
                         f" uv run scripts/mutation-score.py --std --since {base} --min 1.0")
        return "std mutants part"

    def _slices(self, slices: list[str]) -> None:
        """The touched paths' owner-named tests, ahead of the rest; one over 20 s fails."""
        if not slices:
            return
        # A separate step, so the hook's 25 s cut during a cold build allows the commit:
        # only a slice's tests running out of time refuse it.
        self.step(f"build of the touched-path slices' tests: {' '.join(slices)}")
        if self.call([sys.executable, "scripts/affected.py", "--build-slices", *slices]):
            raise Failed(f"the slices' tests do not build; rerun: uv run scripts/affected.py --build-slices {' '.join(slices)}")
        self.step(f"touched-path slices: {' '.join(slices)} (scripts/affected.py, each under 20 s)")
        if self.call([sys.executable, "scripts/affected.py", "--run-slices", *slices]):
            raise Failed("a touched-path slice failed or took over 20 s; the lines above name it, rerun:"
                         f" uv run scripts/affected.py --run-slices {' '.join(slices)}")

    def _tests(self) -> None:
        """The workspace tests. --full runs SLOW_TESTS (why: beside the list)."""
        slow = slow_filter()
        if self.gate_part == "checks":
            self.step("cargo nextest + doctests")
            print("skipped: the CI test partitions run them (CPL_GATE_PART=checks)")
        elif self.hit("nextest_doc"):
            self.step("cargo nextest + doctests")
            print(CACHED)
        elif self.mode == "land":
            # Not cached: a partial run must not vouch for the whole tree.
            self._slices(self.sel_slices.split())
            self.step(f"cargo nextest of the crates the train reaches: {self.sel_packages or 'none'}")
            if self.sel_packages:
                expr = f"({self.sel_filter}) & not ({slow})"
                if self.sel_slice_filter:
                    expr += f" & not ({self.sel_slice_filter})"
                if self.call(["cargo", "nextest", "run", "--workspace", "--no-fail-fast", "-E", expr]):
                    raise Failed("a test of a crate the train reaches failed; the lines above name it")
                self.step("cargo test --doc of the touched crates")
                pkgs = [a for p in self.sel_packages.split() for a in ("-p", p)]
                if self.call(["cargo", "test", "-q", *pkgs, "--doc"]):
                    raise Failed("a doctest of a touched crate failed; the lines above name it")
        else:
            if self.call(["cargo", "nextest", "--version"], capture_output=True) == 0:
                self.step("cargo nextest run --workspace (without the --full tests)")
                if self.call(["cargo", "nextest", "run", "--workspace", "--no-fail-fast", "-E", f"not ({slow})"]):
                    raise Failed("a workspace test failed; rerun: cargo nextest run --workspace")
                self.step("cargo test --workspace --doc")
                if self.call(["cargo", "test", "-q", "--workspace", "--doc"]):
                    raise Failed("a doctest failed; rerun: cargo test --workspace --doc")
            else:
                self.step("cargo test --workspace (without the --full tests)")
                skips = [a for t in (*SLOW_TESTS, *UNTIMED_TESTS) for a in ("--skip", t)]
                if self.call(["cargo", "test", "--workspace", "--", *skips]):
                    raise Failed("a workspace test failed; rerun: cargo test --workspace")
            self.save("nextest_doc")


def _interrupt(signum, frame):
    raise KeyboardInterrupt


class Failed(Exception):
    """A tail step failed; the message says how to rerun it."""


# The guards every tier runs before it builds: cheap, and each one protects the
# public history or a file no later step would catch.
SECRETS = (
    r"gh[pousr]_[A-Za-z0-9]{36,}", r"github_pat_[A-Za-z0-9_]{40,}", r"AKIA[0-9A-Z]{16}",
    r"xox[abprs]-[A-Za-z0-9-]{10,}", r"sk-(ant-|proj-)?[A-Za-z0-9_-]{32,}", r"AIza[0-9A-Za-z_-]{35}",
    r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
)
CREDENTIAL_PATH = re.compile(r"(^|/)(\.env(\..+)?|id_(rsa|dsa|ecdsa|ed25519)|.+\.(pem|p12|pfx|keystore))$")
HOME_PATH = re.compile(r"/(home|Users)/[A-Za-z0-9_.-]+/")
NPM = "https://registry.npmjs.org/"


def scan_added(paths: list[str], lines: list[str], hosts: list[str], deny: list[str]) -> str | None:
    """Why a change's added `lines` (and changed `paths`) must not be published, or None.
    `hosts` are scripts/public-hosts.txt's patterns; `deny` is the machine's own
    .git/info/sensitive-words. A blank line in either matches nothing."""
    if found := [p for p in paths if re.search(r"(^|/)dist/", p)]:
        return "\n".join(found) + "\nthe files above are build output under dist/; never commit them"
    if found := [p for p in paths if CREDENTIAL_PATH.search(p)]:
        return "\n".join(found) + "\nthe files above look like credentials; never commit them"
    found = [m.group(0)[:12] + "..." for line in lines for s in SECRETS for m in re.finditer(s, line)]
    if found:
        return "\n".join(found) + "\nadded lines hold what look like secrets (starting as above); remove them"
    public = [h for h in hosts if h.strip() and not h.startswith("#")]
    found = sorted({h for line in lines for h in re.findall(r"https?://([A-Za-z0-9.-]+)", line)
                    if not any(re.fullmatch(f"(?:{p})", h) for p in public)})
    if found:
        return ("\n".join(found) + "\nadded lines link to the hosts above, which are not in scripts/public-hosts.txt;"
                " the history will be public, so remove the links or add a public host to that list")
    if found := [line for line in lines if HOME_PATH.search(line)][:3]:
        return "\n".join(found) + "\nadded lines hold personal home paths; use paths relative to the repository"
    words = [w.lower() for w in deny if w.strip()]
    if found := [line[:60] for line in lines if any(w in line.lower() for w in words)][:3]:
        return "\n".join(found) + "\nadded lines match the local list in .git/info/sensitive-words; remove those words"
    return None


def _git(root: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], cwd=root, capture_output=True, text=True)


def guard(mode: str, root: Path = ROOT) -> int:
    """check.sh's guards before the build; prints why one fails and returns 1."""
    try:
        _guard(mode, root)
    except Failed as e:
        print(f"check: {e}", file=sys.stderr)
        return 1
    return 0


def _guard(mode: str, root: Path) -> None:
    marks = _git(root, "grep", "-nE", r"^(<{7}|>{7}|\|{7})( |$)", "--", ":!*.png", ":!*.gz").stdout.strip()
    if marks:
        raise Failed(f"merge conflict markers left in:\n{marks}")
    # Every test program has its saved trace: trace_goldens fails without one, and the
    # commit tier can stop before it runs. A commit answers only for the programs it
    # adds or changes, so one already on main blocks no lane.
    programs = "crates/cpl-cli/tests/programs/*.cp"
    listed = (_git(root, "diff", "--cached", "--name-only", "--diff-filter=AM", "--", programs) if mode == "commit"
              else _git(root, "ls-files", programs)).stdout.split()
    if untraced := [p for p in listed if not (root / p).with_suffix(".trace").exists()]:
        raise Failed("test programs with no saved trace; record them with\nCPL_BLESS=1 cargo nextest run -p cpl-cli"
                     " --test trace_goldens, read the .trace, commit it:\n" + "\n".join(untraced))
    # Corpus tests are generated by each problem's gen.py and tests.toml, never
    # committed (problem-packages/R9); corpus_test_files counts the ones still in the tree.
    if mode == "commit":
        added = _git(root, "diff", "--cached", "--name-only", "--diff-filter=A", "--", "corpus/problems/*/tests/*").stdout.split()
        if added:
            raise Failed("new files in a problem's tests/ folder; list the case in the problem's tests.toml and let"
                         " gen.py make it (problem-packages/R9):\n" + "\n".join(added))

    # Staged changes, plus commits not on origin/main yet, so a bot that has already
    # committed is checked before it pushes.
    print("\n==> staged files: no dist/ output, no secrets", flush=True)
    ranges = ["--cached"] + (["origin/main...HEAD"] if _git(root, "rev-parse", "-q", "--verify", "origin/main").returncode == 0 else [])
    hosts = (root / "scripts/public-hosts.txt").read_text(encoding="utf-8").splitlines()
    deny_file = Path(_git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.strip()) / "info/sensitive-words"
    deny = deny_file.read_text(encoding="utf-8", errors="replace").splitlines() if deny_file.is_file() else []
    for r in ranges:
        paths = _git(root, "diff", r, "--name-only", "--diff-filter=ACMR").stdout.splitlines()
        diff = _git(root, "diff", r, "--diff-filter=ACMR", "-U0", "--no-color").stdout.splitlines()
        lines = [l for l in diff if l.startswith("+") and not l.startswith("++")]
        if why := scan_added(paths, lines, hosts, deny):
            raise Failed(why)

    # npm writes the registry it installed from into the lockfile; only the public
    # registry's URLs belong in the repository.
    print("\n==> lockfiles: packages resolve from the public npm registry", flush=True)
    resolved = _git(root, "grep", "-h", "-E", '"resolved": "', "--", "*package-lock.json").stdout.splitlines()
    if private := [l for l in resolved if f'"resolved": "{NPM}' not in l][:3]:
        raise Failed("\n".join(private) + f"\na lockfile resolves packages from a private mirror; rewrite its URLs to {NPM}")



TIMING_EXTRA = ("every_corpus_program_and_conformance_case_is_served", "every_port_compiles_with_every_judges_flags")
FAST_BUDGET_S = 180
AUDIT_FIRST = "staged corpus packages: header (scripts/corpus.py verify --only header)"


# The commit tier's steps, in the order Tail._commit runs them, by the names
# `ci.py commit --only` takes. A hook cut past 25 s leaves the ones it didn't
# reach to land.sh (CPL_COMMIT_PROGRESS); the touched-path slices and the staged
# conformance cases have pending files of their own, and gen.py --check only notes.
COMMIT_STEPS = ("scripts", "fmt", "clippy", "wasm", "nextest", "data", "check_files", "audit", "idioms", "traits",
                "touched_steps", "script_tests", "site")


def selection(mode: str, run=subprocess.run) -> dict[str, str]:
    """scripts/affected.py's touched-path selection: --land diffs the train against
    CHECK_BASE (default origin/main), --commit the staged files."""
    argv = [sys.executable, "scripts/affected.py"]
    if mode == "land":
        base = os.environ.get("CHECK_BASE") or run(["git", "rev-parse", "-q", "--verify", "origin/main"], cwd=ROOT,
                                                   capture_output=True, text=True).stdout.strip()
        if not base:
            return {"all": "1", "reason": "no base to diff the train against"}
        argv += ["--base", base]
    done = run(argv, cwd=ROOT, capture_output=True, text=True)
    if done.returncode:
        raise Failed(f"scripts/affected.py failed; rerun: uv run {' '.join(argv[1:])}\n{done.stderr}")
    return dict(line.split("=", 1) for line in done.stdout.splitlines() if "=" in line)


def run_gate(mode: str, part: str = "", cpl: str = "target/debug/cpl", cache_dir: str = "", manifest: str = "",
             since: float | None = None, run=subprocess.run, tail_cls=None, clock=time.time,
             only: frozenset[str] | None = None) -> int:
    """Every tier after check.sh's guards; `since` is when the gate started. `only`
    runs just those of the commit tier's COMMIT_STEPS (land.sh, after a hook's cut)."""
    tail_cls = tail_cls or Tail
    since = clock() if since is None else since
    times = [(int(clock() - since), "setup")]
    # Tests that mix a timing budget into a correctness check skip the budget.
    if mode not in ("full", "timing"):
        os.environ["CPL_NO_WALL_CLOCK"] = "1"
    try:
        g = gate_part(part, mode)
        if mode == "timing":
            print("\n==> wall-clock budgets (reported, never blocking)", flush=True)
            expr = " or ".join([wall_filter(), *(f"test(={t})" for t in TIMING_EXTRA)])
            walls = run(["cargo", "nextest", "run", "--workspace", "--no-fail-fast", "-E", expr], cwd=ROOT).returncode
            print("\n==> every touched-path slice against its limit (reported, never blocking)", flush=True)
            slices = run([sys.executable, "scripts/affected.py", "--run-slices", "all"], cwd=ROOT).returncode
            return walls or slices
        if g["kind"] != "checks":
            return tail_cls(mode, part, cpl, cache_dir, manifest).part(since)
        sel: dict[str, str] = {}
        if mode in ("land", "commit"):
            sel = selection(mode, run)
            if mode == "commit" and sel.get("audit") == "1" and (only is None or "audit" in only):
                # Before anything that builds, --fast included, so the hook's time limit
                # never skips it: header needs no binary. package runs later, with the
                # cpl this tier builds from the tree being committed.
                print(f"\n==> {AUDIT_FIRST}", flush=True)
                t0 = clock()
                if run([sys.executable, "scripts/corpus.py", "verify", "--changed", "staged", "--only", "header"],
                       cwd=ROOT).returncode:
                    raise Failed("a staged corpus package fails scripts/corpus.py verify; the finding above names the mode"
                                 " and the package")
                times.append((int(clock() - t0), AUDIT_FIRST))
            if sel.get("all", "1") == "1":
                if only is not None:
                    print(f"\ncheck: {sel.get('reason', '')} is not mapped by scripts/affected.py; CI runs --fast on the tip")
                    return 0
                print(f"\ncheck: {sel.get('reason', '')} is not mapped by scripts/affected.py; running --fast")
                mode, sel = "fast", {}
    except (Failed, ValueError) as e:
        print(f"check: {e}", file=sys.stderr)
        return 1
    if mode == "commit":
        kw = {"only": only} if only is not None else {}
        if (progress := os.environ.get("CPL_COMMIT_PROGRESS")) and only is None:
            kw["progress"] = progress
            Path(progress).write_text("\n".join(COMMIT_STEPS) + "\n")
        tail = tail_cls("commit", cpl=cpl, run=run, **kw)
        code = tail.commit(sel)
    else:
        tail = tail_cls(mode, part, cpl, cache_dir, manifest, sel.get("packages", ""), sel.get("filter", ""),
                        sel.get("site", ""), sel_slices=sel.get("slices", ""),
                        sel_slice_filter=sel.get("slice_filter", ""))
        code = tail()
    if code:
        return 1
    print("\nstep times (s):")
    for secs, title in sorted(times + tail.times, key=lambda t: -t[0]):
        if secs > 0:
            print(f"{secs:5d}  {title}")
    elapsed = int(clock() - since)
    if mode == "commit":
        print(f"\ncheck: commit-tier checks passed in {elapsed}s; CI runs --fast on every main tip")
    elif mode in ("fast", "land"):
        record = target_dir() / "gate-fast-time.json"
        record.parent.mkdir(parents=True, exist_ok=True)
        record.write_text(f'{{"mode":"{mode}","elapsed_s":{elapsed},"budget_s":{FAST_BUDGET_S}}}\n')
        print(f"\ncheck: all {mode} checks passed in {elapsed}s (budget <= {FAST_BUDGET_S}s)")
        # Wall time depends on the machine and on how warm the build is, so an overrun
        # warns and stays recorded in the scoreboard instead of failing.
        if elapsed > FAST_BUDGET_S:
            print(f"check: WARNING: fast gate took {elapsed}s, over the {FAST_BUDGET_S}s (3 min) budget")
    else:
        print(f"\ncheck: all {mode} checks passed in {elapsed}s")
    return 0

CHECK_FLAGS = {"--fast": "fast", "--commit": "commit", "--land": "land", "--full": "full", "--timing": "timing"}
CXX_CACHE_MAX_AGE_S = 8 * 86400


def cxx_cache(target: Path, env, run=subprocess.run, clock=time.time) -> None:
    """Point g++ at scripts/cxx-cache.py, which reuses a binary built from the same
    source, flags and g++, and drop cached binaries unused for over a week."""
    if env.get("CHECK_NO_CXX_CACHE", "0") == "1" or not shutil.which("g++", path=env.get("PATH")):
        return
    shim = target / "cxx-shim"
    home_cache = Path(env.get("XDG_CACHE_HOME") or Path(env.get("HOME") or Path.home()) / ".cache")
    cache = Path(env.get("CPL_CXX_CACHE_DIR") or home_cache / "cpl" / "cxx")
    shim.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)
    link = shim / "g++"
    if link.is_symlink() or link.exists():
        link.unlink()
    link.symlink_to(ROOT / "scripts" / "cxx-cache.py")
    env.update(CPL_CXX_SHIM_DIR=str(shim), CPL_CXX_CACHE_DIR=str(cache),
               PATH=f"{shim}{os.pathsep}{env.get('PATH', '')}")
    key = run(["g++", "--cpl-system-key"], env=dict(env), capture_output=True, text=True)
    env["CPL_CXX_SYSTEM_KEY"] = key.stdout.rstrip("\n") if key.returncode == 0 else ""
    probes = target / "judge-probes"
    env["CPL_JUDGE_PROBE_DIR"] = str(probes)
    shutil.rmtree(probes, ignore_errors=True)
    now = clock()
    for f in cache.rglob("*"):
        if f.is_file() and not f.is_symlink() and now - f.stat().st_mtime >= CXX_CACHE_MAX_AGE_S:
            f.unlink()


def cxx_trim(cache: Path, since: float) -> None:
    """Drop the cached binaries not used since `since`, before CI saves the cache. A job
    with no cache of its own restores another's, so without this every gate part's cache
    grew to all of theirs: 9 parts of 460 MB a run, past GitHub's 10 GB, which then
    evicts live caches."""
    for f in cache.rglob("*"):
        if f.is_file() and not f.is_symlink() and f.suffix != ".err" and f.stat().st_mtime < since:
            f.unlink()
            f.with_suffix(".err").unlink(missing_ok=True)


def check(argv: list[str], env=None, run=subprocess.run, clock=time.time, guard_fn=None, gate_fn=None) -> int:
    """scripts/check.sh: parse its flags, set up the C++ and stage caches, run the
    guards, then the tier (`run_gate`)."""
    env = os.environ if env is None else env
    since = clock()
    target = target_dir()
    (target / "gate-fast-time.json").unlink(missing_ok=True)
    mode, use_cache = "fast", env.get("CHECK_NO_CACHE", "0") != "1"
    for arg in argv:
        if arg in CHECK_FLAGS:
            mode = CHECK_FLAGS[arg]
        elif arg == "--no-cache":
            use_cache = False
        elif arg in ("-h", "--help"):
            head = (ROOT / "scripts" / "check.sh").read_text(encoding="utf-8").splitlines()[1:25]
            print("\n".join(re.sub(r"^# ?", "", line) for line in head if line.startswith("#")))
            return 0
        else:
            print(f"check: unknown argument `{arg}`", file=sys.stderr)
            return 2
    use_cache = use_cache and mode != "full"
    cache_dir = target / "check-cache"
    if mode in ("fast", "land"):
        cxx_cache(target, env, run, clock)
    manifest = cache_dir / f"manifest.{os.getpid()}"
    try:
        if use_cache:
            cache_dir.mkdir(parents=True, exist_ok=True)
            use_cache = run([sys.executable, "scripts/check_cache.py", "manifest", str(manifest)],
                            cwd=ROOT, env=dict(env)).returncode == 0
        if (guard_fn or guard)(mode):
            return 1
        if not shutil.which("cargo", path=env.get("PATH")):
            print("check: `cargo` is not installed; see https://rustup.rs", file=sys.stderr)
            return 1
        return (gate_fn or run_gate)(mode, env.get("CPL_GATE_PART", ""), str(target / "debug" / "cpl"),
                                     str(cache_dir), str(manifest) if use_cache else "", since)
    finally:
        manifest.unlink(missing_ok=True)
        Path(f"{manifest}.reads").unlink(missing_ok=True)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    gc = sub.add_parser("commit", help="the commit tier's named steps on the staged files (land.sh, after a hook's cut)")
    gc.add_argument("--only", required=True, help=f"comma-separated, of: {','.join(COMMIT_STEPS)}")
    gc.add_argument("--cpl", default="target/debug/cpl")
    g = sub.add_parser("steps", help="run a tier's checking scripts, spec gates among them")
    g.add_argument("tier", choices=sorted(STEPS) + ["touched"])
    g.add_argument("--cpl", default="target/debug/cpl")
    g.add_argument("scripts", nargs="*", help="with tier `touched`: the changed scripts whose steps run")
    gt = sub.add_parser("gate", help="run a tier after check.sh's guards")
    gt.add_argument("--mode", choices=MODES, required=True)
    gt.add_argument("--gate-part", default="")
    gt.add_argument("--cpl", default="target/debug/cpl")
    gt.add_argument("--cache-dir", default="")
    gt.add_argument("--manifest", default="", help="the stage cache's manifest; empty turns the cache off")
    gt.add_argument("--since", type=float, default=None, help="when the gate started (epoch seconds)")
    sub.add_parser("check", help="scripts/check.sh [--fast|--commit|--land|--full|--timing] [--no-cache]")
    gd = sub.add_parser("guard", help="the guards before the build: conflict markers, traces, secrets, lockfiles")
    gd.add_argument("--mode", choices=MODES, required=True)
    tr = sub.add_parser("cxx-trim", help="drop the g++ cache's binaries unused since --since (epoch seconds)")
    tr.add_argument("--since", type=float, required=True)
    tr.add_argument("--dir", default=str(Path.home() / ".cache" / "cpl" / "cxx"))
    argv = sys.argv[1:] if argv is None else argv
    if argv[:1] == ["check"]:
        return check(argv[1:])
    args = ap.parse_args(argv)
    if args.cmd == "guard":
        return guard(args.mode)
    if args.cmd == "commit":
        only = frozenset(k for k in args.only.split(",") if k)
        if unknown := sorted(only - set(COMMIT_STEPS)):
            ap.error(f"no commit-tier step {', '.join(unknown)}; the steps: {', '.join(COMMIT_STEPS)}")
        return run_gate("commit", cpl=args.cpl, only=only)
    if args.cmd == "cxx-trim":
        return cxx_trim(Path(args.dir), args.since) or 0
    if args.cmd == "gate":
        return run_gate(args.mode, args.gate_part, args.cpl, args.cache_dir, args.manifest, args.since)
    if args.cmd == "steps":
        if (args.tier == "touched") != bool(args.scripts):
            ap.error("name the changed scripts with `touched`, and only with it")
        return run_steps(args.tier, args.cpl, only=set(args.scripts) or None)

if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        # A tier past its time limit (the hook's `timeout`) or Ctrl-C: the caller says which step.
        sys.exit(130)
