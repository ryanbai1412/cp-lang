# Milestone 1 acceptance run

Owner: idioms. This run tests [handbook/product/milestones.md](../../handbook/product/milestones.md)
end to end. A cold agent plays a Codeforces 1500+ friend who has never used CPL. Each
run is one JSON file in `runs/`; `blockers.py` ranks the findings across runs into
`plan/product/BLOCKERS.md`. Domain metrics are means; this run is the end.

## The friend
- **Machine:** a fresh one, with no checkout of this repo, no `skills/cp-lang/`, and no
  earlier CPL install. Every run covers Linux, macOS and Windows. The agent drives one
  OS end to end; CI runners on the others follow the site's install instructions word
  for word and run the CLI parts of `test` and `submit`. Each OS gets its own `os` entry.
- **Editors:** Neovim, set up exactly as the site documents, is the editor for every
  step, plus a plain Vim setup. VS Code and a JetBrains IDE are installed and set up from
  the site's instructions, and each is checked for highlighting, diagnostics, completion,
  format and go-to-definition.
- **Allowed sources:** https://cp-lang.org and every page it links, the installer it
  points to, the judge's problem pages, and general knowledge of C++ and competitive
  programming. Asking a person, reading the repo, or reading these protocol notes is
  not allowed. Using a source outside the list is a finding (`workaround`).
- **Persona:** fluent in C++, solves Div2 A-D reliably, and impatient: stuck on the
  same thing for 10 minutes, it records a `blocker` and takes the cheapest way around.

## The eight steps
Steps 1-7 follow MILESTONE-1.md section 5.1 (journeys from EXPERIENCE.md in brackets).
Step 8 is Ryan's in-depth reader (2026-11-22): his friends are PL-curious and will read
the advanced chapters to understand the model and the reasons behind it.

| Step | Task | Passes when |
|---|---|---|
| `understand` (J1) | Open the site; say in 3 sentences what CPL is for and what it is better at; read the intro guide. | The summary matches VISION.md V1-V3, from the site alone, in under 30 minutes. |
| `try` (J2) | In the browser playground, run and change an example, then solve 3 intro exercises. | 3/3 exercises give correct output in the browser, nothing installed. |
| `install` (J3) | Install the CLI and the Neovim setup from the site's instructions. | On every OS `cpl --version` works. In Neovim a new `.cp` file shows diagnostics, completion, formatting and go-to-definition after the one documented setup step, and Vim gets highlighting. VS Code and JetBrains pass the same five features. |
| `solve` (J4, J5) | Solve the 10 held-out Div2 A-E problems (`problems/div2.txt`), reading statements on the judge. | At least 8/10 accepted by the local judge (`judge.py`): the samples plus 200 small generated cases agree with an exhaustive `brute.py`, or pass `check.py` where answers aren't unique. The `max.py` input must finish within 2x `tl`. |
| `test` (J6, J7) | For each problem, run the samples and a custom test. For one problem, find a planted bug with the documented stress or debug workflow. | Samples and custom tests ran with the CLI; the planted bug was found from the docs. |
| `submit` (J9) | Build every solved problem to C++ the way the docs say. Load samples with Competitive Companion if the site documents it. | The C++ compiles with `g++ -std=c++17 -O2` and passes the local judge. No real judge submissions. |
| `compete` (J5-J9) | A timed virtual Codeforces round: 5 unseen problems (`problems/contest.txt`), 2 hours, no pauses. | At least 4/5 solved in time, and no problem lost to missing ordinary functionality or a tool failure. |
| `design` (J2, advanced) | As a PL-curious reader, read the ownership and mutation chapter and one other in-depth chapter. Predict the result (output, compile error or trap) of 5 probe snippets before running them. | At least 4/5 predictions right, and both chapters explain why each design was chosen, not just what it does. A wrong prediction the docs should have prevented, or a missing reason, is a `blocker`. |

## What the agent records
A run may also carry `fixed` (ids of earlier findings this run confirms fixed) and `ci`
(`run`, `linux`, `macos`, `windows` scores of `ci-steps.sh`). A finding whose latest report has
`"status": "fixed"` (or a summary starting `FIXED`) leaves the ranking for the Fixed list.

One JSON object per run, saved as `runs/<YYYY-MM-DD>-<n>.json`:

```json
{
  "date": "2026-11-23",
  "os": "linux (Ubuntu 22.04); macos and windows install/CLI from CI",
  "site_commit": "<the commit cp-lang.org was serving>",
  "agent": "<model and harness>",
  "steps": {"understand": {"pass": true, "minutes": 12}, "...": {}},
  "problems": [{"id": "cf2xxxB", "set": "div2", "verdict": "AC", "minutes": 18, "attempts": 2}],
  "findings": [{
    "id": "playground-no-stdin",
    "step": "try",
    "kind": "blocker",
    "lane": "site",
    "summary": "the playground has no stdin box, so exercise 2 could not run",
    "minutes_lost": 6,
    "evidence": "the URL, command or snippet that reproduces it"
  }]
}
```

- `steps` has all eight keys, each with `pass` and `minutes` (wall clock).
- `kind`: `blocker` (the step could not pass without it), `workaround` (it passed
  another way, such as a source outside the list or rewriting around a gap),
  `non-idiomatic` (code the language pushed the friend into: an index juggle, a manual
  loop for a missing library call, a type fight, a duplicated fact; quote it and what
  they wanted to write), or `hesitation` (unsure for a minute or more about how CPL
  says something).
- The score is language quality, not pass/fail (Ryan, 2026-11-22). `BLOCKERS.md` ranks by
  kind, then runs seen in, then minutes lost, and reports workarounds, non-idiomatic
  code and hesitations per accepted problem.
- `lane` is a folder under `plan/domains/`. `id` is a stable slug, so the same
  finding in later runs merges into one row; reuse ids from `BLOCKERS.md`.
- Every finding needs evidence a lane can reproduce without the transcript.

## Milestone 2 runs
[MILESTONE-2](../../handbook/product/milestones.md) runs the same eight steps on each release
candidate, on Linux, macOS and Windows, with these changes:
- **A fresh held-out set per run.** `problems/sets/<set>.txt` lists 10 Div2, 5 Div1 and a
  5-problem timed round, each newer than the corpus's newest contest and never used by an
  earlier run, with a local judge in `problems/<id>/` (plus `ref.cpp`, a fast reference
  that must be AC under `judge.py`). A problem that a run has used is no longer held out;
  the next run takes the next set. The first is `m2-1`.
- **Four personas, one agent each per OS,** all under the friend's rules above:

| Persona | Who | Steps and sets | Passes when |
|---|---|---|---|
| `beginner` | Knows basic C++ or Python, solves Div2 A-B, never used a language server. | All 8; `solve` on the set's first 5 Div2. | 4/5 `solve`, 3/5 `compete`; every step reached from the site's beginner path. |
| `cf1500` | The Milestone 1 friend. | All 8; `solve` on the 10 Div2. | As in Milestone 1. |
| `elite` | Codeforces 2400+, fluent in tuned C++, judges by speed and control. | `install`, `solve` on the 5 Div1, `submit`, `compete`. | 4/5 Div1 accepted, each max input within 1.5x of `ref.cpp`, and one solution tuned to within 1.1x from the docs alone. |
| `reader` | PL-curious, reads the advanced chapters. | `understand`, `try`, `design` with 10 probes. | 9/10 predictions, every chapter gives its reasons. |

- **Gate:** stable is promoted only when every persona passes every step it runs on every
  OS and BLOCKERS.md has no open P0 or blocker row.

## After a run
1. Add the JSON, run `research/acceptance/blockers.py`, and commit both with the
   run's transcript summary in `runs/<date>-<n>.md`.
2. Copy the judged solutions to `solutions/<run>/<os>/<id>.cp`. A draft that exposed a
   blocker goes to `open-repros/<run>/<id>-<what>.cp` while the blocker is open, and moves to
   `solutions/<run>/repro/` once its fix lands and the draft is AC. CI's `acceptance` job
   runs `regress.py`, which rebuilds everything under `solutions/` with `--judge codeforces`
   and judges it, so anything there that is not AC turns main red. A blocker is marked fixed
   (`"status": "fixed"` plus a `recheck` note) only once its repro is AC.
3. Append each lane's new or changed rows to its `worklog.md` as a request.
4. Record the headline (steps passed, minutes per step) in the idioms `metrics.toml`.

Cadence: a run every 2 hours until the milestone passes (due about 09:30 PST
2026-11-23), each followed by the step score and the ranked list in the idioms worklog.
