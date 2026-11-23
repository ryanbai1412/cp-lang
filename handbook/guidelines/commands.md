# Writing commands and editor features

| | |
|---|---|
| Kind | Guideline |
| Owner | tools |
| Applies to | `crates/cpl-cli/`, `crates/cpl-contest/`, `crates/cpl-lsp/`, `crates/cpl-wasm/` (the engine calls), `editors/`, `spec/tools/` |

Read this before you add or change a `cpl` command, a flag, an editor
feature or a playground call. It explains what a contestant should feel
when they use our tools, and the few ideas that get us there. Every rule,
with its reason and its check, is enumerated in
`handbook/guidelines/commands-checklist.md`.

## A good command, by example

```
$ cpl test a.cp
a.cp: 3 cases
  1  ok   12 ms
  2  WA   9 ms   line 1: expected 7, got 6
  3  ok   11 ms
2 of 3 pass
$ echo $?
3
```

It says what it ran, one line per case, the first difference in the
contestant's terms, and a summary. The exit code says the outcome, so a
script or an editor acts on it without reading the text. `cpl test a.cp
--json` prints the same facts as one JSON object per case, which the VS
Code and JetBrains test panels read.

A bad command prints a Rust panic for a missing file, takes a second to
show `--help`, spells the same flag `--format=json` here and `--json`
there, or has an editor button that re-implements it and slowly drifts
away from what the terminal says.

## The ideas

**One function per job, many faces.** A command is a plain function
that takes its arguments and two output streams and returns an exit code.
The terminal, the playground and the tests all call that function. Each job
the editor also does (diagnostics, fixes, running, tracing, finding a
problem's cases) likewise has one function, and the language server and
the editor packages only present its result. When a second caller is
added, a test checks that both callers give the same answer. This is what
makes "the editor and the terminal disagree" impossible rather than
unlikely.

**Ask the compiler, never guess.** Types, ranges, resolved names and costs
come from the compiler's analysis of the file. A tool that guesses from the
text is right on the examples and wrong on the contestant's file.

**One vocabulary.** A flag learned on one command works on the next; the
spec, `--help`, errors and hover use the same words. `spec/tools/cli.md`
and the binary list the same commands.

**Machine output is a contract.** Editors and scripts parse `--json` and
the exit codes, so field names, order and codes only change through the
spec.

**Instant, and never broken.** Every command starts in well under a tenth
of a second and every editor request answers within its budget, on all
three operating systems. The contestant is always mid-edit, so half-typed
code gets answers, never an internal error.

**Measure what contestants type.** Completion, hover and go to definition
are checked by replaying every corpus solution, not only hand-picked
examples, and each measure is a ratchet that only goes up.

## Deciding a case nobody wrote down

Ask what the contestant at 3am would expect from the command's name and
from its neighbours, and make the terminal, the editor and the playground
give that answer through one function. If two of them would need different
code, the job is missing its shared function.

# Reference

The rules, each with an identifier the checks cite, are in
`handbook/guidelines/commands-checklist.md`. The architecture they protect
is `handbook/design/editor.md`. Tools' quality register,
`handbook/domains/tools.md`, lists every quality and its check.

## Changes

- 2026-11-23: first version.
