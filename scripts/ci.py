#!/usr/bin/env python3
"""The CI driver's decisions, tested (scripts/test_ci.py): which tests each tier
leaves out, what a CPL_GATE_PART runs, and which spec gates each tier runs.
scripts/check.sh evals `ci.py plan` and runs `ci.py gates <tier>`.

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


SPEC_GATE = Path(__file__).resolve().parent / "spec-gate.py"


@dataclass(frozen=True)
class Gate:
    """One `spec-gate.py` run and what check says when it fails. A `derived`
    gate checks generated files the regen job rewrites, so it only warns once
    CPL_DERIVED_HEALED=1; a `quiet` one shows its output only on failure."""
    args: tuple[str, ...]
    fail: str
    derived: bool = False
    quiet: bool = False


def _stale(what: str, gate: str) -> str:
    return f"{what} is stale; run: uv run scripts/spec-gate.py {gate} --write"


# Every spec and conformance gate check.sh runs, by tier, through scripts/spec-gate.py.
GATES = {
    "checks": (
        Gate(("coverage",), "rule coverage gate failed; see above, then rerun: uv run scripts/spec-gate.py coverage",
             derived=True),
        Gate(("subset",), _stale("spec/SUBSET.md", "subset")),
        Gate(("matrix",), "matrix gate failed; see above"),
        Gate(("widths",), _stale("spec/conformance/widths/", "widths")),
        Gate(("ownership",), _stale("spec/conformance/ownership/", "ownership")),
        Gate(("xref",), "a rule id names no rule in spec/core; see above"),
        Gate(("changelog",), "a spelling the spec removes has no changelog line; see above"),
        Gate(("tool-rules",), "a tool feature in spec/tools/ has no rule id; see above"),
        Gate(("std-signatures",), "a spec/lib chapter and std disagree on a return type; see above"),
        Gate(("decisions-explained",), "a decision cited in spec/ has no handbook explanation; see above"),
        Gate(("traits",), "trait gate failed; see above"),
        Gate(("sigs",), "sig gate failed; see above"),
        Gate(("prelude",), "prelude gate failed; see above"),
        Gate(("std-docs", "--audit"), "a public std fn lacks a required doc label; see above", quiet=True),
        Gate(("vocab",), "a spelling spec/GLOSSARY.md bans; see above"),
    ),
    "full": (
        Gate(("metamorphic",), "a meaning-preserving edit changed a case's output; see above"),
        Gate(("speed",), "the conformance and spec-block suites went over their CPU budget; see above"),
        Gate(("explain-copies", "--max", "1"), "cpl explain claims more copies than the C++ makes in more cases; see above"),
        Gate(("tool-vocab",), "help, explain or hover uses a spelling spec/GLOSSARY.md bans; see above"),
        Gate(("flakes", "--runs", "2"), "a conformance case changed verdict between runs; see above"),
    ),
}


def run_gates(tier: str, run=subprocess.run, healed: bool | None = None) -> int:
    """Runs the tier's gates in order and stops at the first that fails."""
    if healed is None:
        healed = os.environ.get("CPL_DERIVED_HEALED") == "1"
    for gate in GATES[tier]:
        print(f"\n==> spec-gate.py {' '.join(gate.args)}", flush=True)
        done = run([sys.executable, str(SPEC_GATE), *gate.args], capture_output=gate.quiet, text=True)
        if done.returncode == 0:
            continue
        if gate.quiet:
            sys.stdout.write(done.stdout)
            sys.stderr.write(done.stderr)
        if gate.derived and healed:
            print(f"warning: {gate.fail} (the regen job commits the fix)", file=sys.stderr)
            continue
        print(f"check: {gate.fail}", file=sys.stderr)
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("plan", help="print the tier's filters and gate part as shell assignments")
    p.add_argument("--mode", choices=MODES, required=True)
    p.add_argument("--gate-part", default="")
    g = sub.add_parser("gates", help="run a tier's spec gates (scripts/spec-gate.py)")
    g.add_argument("tier", choices=sorted(GATES))
    args = ap.parse_args(argv)
    if args.cmd == "gates":
        return run_gates(args.tier)
    try:
        sys.stdout.write(plan(args.mode, args.gate_part))
    except ValueError as e:
        print(f"check: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
