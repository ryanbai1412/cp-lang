#!/usr/bin/env python3
"""The CI driver's decisions, tested (scripts/test_ci.py): which tests each tier
leaves out, what a CPL_GATE_PART runs, and which checking scripts and spec gates
each tier runs, and the --fast tier from its build on. scripts/check.sh evals
`ci.py plan`, runs `ci.py steps <tier>` and hands the rest to `ci.py tail`.

CI splits --fast across parallel jobs (.github/workflows/ci.yml):
  checks            everything but the workspace tests, the conformance ratchet and fmt
  fmt               fmt_check over every .cp file
  conform[:S+S@N/M] the conformance ratchet (only suites S, shard N/M) and, with the
                    conformance suite, the script unit tests
  tests:doc         the doctests
  tests:<partition> that nextest partition (e.g. tests:hash:1/12)
Unset runs all of --fast.
"""
from __future__ import annotations

import argparse
import os
import re
import shlex
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
    if part == "fmt" or part == "conform" or part.startswith(("conform:", "tests:")):
        if mode != "fast":
            raise ValueError(f"CPL_GATE_PART={part} needs --fast")
    if part == "fmt":
        out["kind"] = "fmt"
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
        raise ValueError(f"unknown CPL_GATE_PART `{part}` (checks, fmt, conform or tests:<partition>)")
    return out


def plan(mode: str, part: str) -> str:
    """Shell assignments check.sh evals."""
    g = gate_part(part, mode)
    values = {
        "slow_filter": slow_filter(),
        "wall_filter": wall_filter(),
        "gate_kind": g["kind"],
        "conform_suites": g["suites"],
        "conform_shard": g["shard"],
        "part": g["partition"],
    }
    arrays = {"slow_tests": SLOW_TESTS}
    return ("".join(f"{k}={shlex.quote(v)}\n" for k, v in values.items())
            + "".join(f"{k}=({' '.join(map(shlex.quote, v))})\n" for k, v in arrays.items()))


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
USECASES = Step("use cases: unique IDs, every field, evidence that exists (scripts/check-usecases.py)",
                ("scripts/check-usecases.py", "--check"), "use-case catalogue check failed; see above", stdout=False)

# Every checking script check.sh runs, by tier, in order: `commit` before a
# commit, `checks` after --fast's build, `full` (with --full's slow suites) last.
STEPS = {
    "commit": (RATCHET, USECASES),
    "checks": (
        RATCHET,
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
        Step("g++ -O2 compile time of the bench programs within its bars (bench/compile_time.py --check)",
             ("bench/compile_time.py", "--check", "--cpl", "{cpl}"),
             "the bench programs' g++ -O2 time is over a bar in bench/compile_time.py; see above"),
        Step("every std fn with a case grows as its cost: says (bench/scaling.py --check)",
             ("bench/scaling.py", "--check", "--cpl", "{cpl}"),
             "a std fn's time at n and 4n disagrees with its cost:, or research/perf/scaling.json is stale; see above"),
        gate("coverage", fail="rule coverage gate failed; see above, then rerun: uv run scripts/spec-gate.py coverage",
             derived=True),
        gate("subset", fail=_stale("spec/SUBSET.md", "subset")),
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
        gate("explain-copies", "--max", "1",
             fail="cpl explain claims more copies than the C++ makes in more cases; see above"),
        gate("tool-vocab", fail="help, explain or hover uses a spelling spec/GLOSSARY.md bans; see above"),
        gate("flakes", "--runs", "2", fail="a conformance case changed verdict between runs; see above"),
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
              only=None) -> int:
    """Runs the tier's steps, or with `only` the touched steps running those scripts, in order,
    and stops at the first that fails."""
    if healed is None:
        healed = os.environ.get("CPL_DERIVED_HEALED") == "1"
    with tempfile.TemporaryDirectory(prefix="cpl-ci-") as tmp:
        values = {"cpl": cpl, "tmp": tmp, "nproc": str(os.cpu_count() or 1)}
        for step in touched_steps(only) if only else STEPS[tier]:
            print(f"\n==> {step.title}", flush=True)
            missing = [c for c in step.needs if not shutil.which(c)]
            if missing:
                print(f"check: `{missing[0]}` is not installed; {step.title} needs it", file=sys.stderr)
                return 1
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
            return 1
    return 0


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
        # A killed tier stops its backgrounded jobs too: SIGTERM unwinds like Ctrl-C.
        signal.signal(signal.SIGTERM, _interrupt)
        with tempfile.TemporaryDirectory(prefix="cpl-tail-") as logs:
            try:
                self._tier(logs)
            except Failed as e:
                print(f"check: {e}", file=sys.stderr)
                return 1
            finally:
                self.stop_all()
                self.end_step()
        return 0

    def _tier(self, logs: str) -> None:
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

        self.step("every corpus package passes the static modes (scripts/corpus.py verify --no-run)")
        if self.call([sys.executable, "scripts/corpus.py", "verify", "--no-run", "--only",
                      "header,check,package,lint", "--cpl", self.cpl]):
            raise Failed("a corpus package fails scripts/corpus.py verify; the finding above names the mode and the package")
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

    def part(self, since: float) -> int:
        """A CI gate part other than checks (CPL_GATE_PART=conform..., tests:..., fmt);
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
                if self.call([sys.executable, "scripts/corpus.py", "make"], stdout=subprocess.DEVNULL):
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
        raise ValueError(f"CPL_GATE_PART={self.gate_part} runs through `ci.py tail`")

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
            self.step(f"cargo nextest of the crates the train reaches: {self.sel_packages or 'none'}")
            if self.sel_packages:
                if self.call(["cargo", "nextest", "run", "--workspace", "--no-fail-fast",
                              "-E", f"({self.sel_filter}) & not ({slow})"]):
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


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("plan", help="print the tier's filters and gate part as shell assignments")
    p.add_argument("--mode", choices=MODES, required=True)
    p.add_argument("--gate-part", default="")
    g = sub.add_parser("steps", help="run a tier's checking scripts, spec gates among them")
    g.add_argument("tier", choices=sorted(STEPS) + ["touched"])
    g.add_argument("--cpl", default="target/debug/cpl")
    g.add_argument("scripts", nargs="*", help="with tier `touched`: the changed scripts whose steps run")
    t = sub.add_parser("tail", help="run the --fast tier from its build on: the tests and what overlaps them")
    t.add_argument("--mode", choices=MODES, required=True)
    t.add_argument("--gate-part", default="")
    t.add_argument("--cpl", default="target/debug/cpl")
    t.add_argument("--cache-dir", default="")
    t.add_argument("--manifest", default="", help="the stage cache's manifest; empty turns the cache off")
    t.add_argument("--sel-packages", default="")
    t.add_argument("--sel-filter", default="")
    t.add_argument("--sel-site", default="")
    t.add_argument("--times", help="append each step's `seconds<TAB>title` here")
    pt = sub.add_parser("part", help="run a CI gate part other than checks (CPL_GATE_PART)")
    pt.add_argument("--mode", choices=MODES, required=True)
    pt.add_argument("--gate-part", required=True)
    pt.add_argument("--cpl", default="target/debug/cpl")
    pt.add_argument("--cache-dir", default="")
    pt.add_argument("--manifest", default="", help="the stage cache's manifest; empty turns the cache off")
    pt.add_argument("--since", type=float, default=time.time(), help="when the gate started (epoch seconds)")
    args = ap.parse_args(argv)
    if args.cmd == "part":
        return Tail(args.mode, args.gate_part, args.cpl, args.cache_dir, args.manifest).part(args.since)
    if args.cmd == "tail":
        tail = Tail(args.mode, args.gate_part, args.cpl, args.cache_dir, args.manifest,
                    args.sel_packages, args.sel_filter, args.sel_site)
        code = tail()
        if args.times:
            with open(args.times, "a", encoding="utf-8") as f:
                f.writelines(f"{secs}\t{title}\n" for secs, title in tail.times)
        return code
    if args.cmd == "steps":
        if (args.tier == "touched") != bool(args.scripts):
            ap.error("name the changed scripts with `touched`, and only with it")
        return run_steps(args.tier, args.cpl, only=set(args.scripts) or None)
    try:
        sys.stdout.write(plan(args.mode, args.gate_part))
    except ValueError as e:
        print(f"check: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
