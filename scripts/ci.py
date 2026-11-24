#!/usr/bin/env python3
"""The CI driver's decisions, tested (scripts/test_ci.py): which tests each tier
leaves out, what a CPL_GATE_PART runs, and which checking scripts and spec gates
each tier runs. scripts/check.sh evals `ci.py plan` and runs `ci.py steps <tier>`.

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
import shlex
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

# Which --fast leaves to --full (the reasons are in scripts/check.sh's nextest step).
SLOW_TESTS = (
    "products_and_variable_divisors_are_never_decided_wrongly",
    "front_end_check_is_under_50_ms",
    "recovery_scoreboard",
    "fix_it_scoreboard",
    "committed_scoreboard_ratchet",
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
    """One checking script check.sh runs and what it says when the script fails.
    `argv` starts with the script's repo path; `{cpl}` is the debug cpl binary.
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
# commit, `checks` after --fast's build, `full` after --full's.
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
             ("scripts/std-trait-matrix.py", "--max", "76"), "a std container lacks a common trait; see above"),
        Step("std docs follow the std-docs checklist (scripts/std-docs-lint.py)",
             ("scripts/std-docs-lint.py", "--max", "0"), "a std doc breaks a std-docs rule; see above"),
        Step("std signatures follow the std-api checklist (scripts/std-api-lint.py)",
             ("scripts/std-api-lint.py", "--max", "0"), "a std signature breaks a std-api rule; see above"),
        Step("std model cases match their brute force on both engines (scripts/std-models.py)",
             ("scripts/std-models.py",),
             "a std type disagrees with its model; the line above gives the shortest failing op sequence"),
        Step("every inventory spelling is a documented std name (scripts/std-inventory.py)",
             ("scripts/std-inventory.py",), "spec/lib/inventory.md names something cpl doc does not document; see above"),
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
        gate("metamorphic", fail="a meaning-preserving edit changed a case's output; see above"),
        gate("speed", fail="the conformance and spec-block suites went over their CPU budget; see above"),
        gate("explain-copies", "--max", "1",
             fail="cpl explain claims more copies than the C++ makes in more cases; see above"),
        gate("tool-vocab", fail="help, explain or hover uses a spelling spec/GLOSSARY.md bans; see above"),
        gate("flakes", "--runs", "2", fail="a conformance case changed verdict between runs; see above"),
    ),
}


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


def run_steps(tier: str, cpl: str = "target/debug/cpl", run=subprocess.run, healed: bool | None = None,
              only=None) -> int:
    """Runs the tier's steps, or with `only` the touched steps running those scripts, in order,
    and stops at the first that fails."""
    if healed is None:
        healed = os.environ.get("CPL_DERIVED_HEALED") == "1"
    for step in touched_steps(only) if only else STEPS[tier]:
        print(f"\n==> {step.title}", flush=True)
        script, *args = (a.replace("{cpl}", cpl) for a in step.argv)
        cmd = [sys.executable, str(ROOT / script), *args]
        if step.drop:
            done = run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            sys.stdout.writelines(l for l in done.stdout.splitlines(True) if step.drop not in l)
        elif step.quiet:
            done = run(cmd, capture_output=True, text=True)
            if done.returncode:
                sys.stdout.write(done.stdout)
                sys.stderr.write(done.stderr)
        else:
            done = run(cmd, stdout=None if step.stdout else subprocess.DEVNULL, text=True)
        sys.stdout.flush()
        if done.returncode == 0:
            continue
        if step.derived and healed:
            print(f"warning: {step.fail} (the regen job commits the fix)", file=sys.stderr)
            continue
        print(f"check: {step.fail}", file=sys.stderr)
        return 1
    return 0


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
    args = ap.parse_args(argv)
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
