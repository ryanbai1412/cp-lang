# Milestone 1 acceptance run

Owner: idioms. This run tests [plan/product/MILESTONE-1.md](../../plan/product/MILESTONE-1.md)
end to end. A cold agent plays a Codeforces 1500+ friend who has never used CPL. Each
run is one JSON file in `runs/`; `blockers.py` ranks the findings across runs into
`plan/product/BLOCKERS.md`. Domain metrics are means; this run is the end.

## The friend
- **Machine:** a fresh one, with no checkout of this repo, no `skills/cp-lang/`, and no
  earlier CPL install. Every run covers Linux, macOS and Windows. The agent drives one
  OS end to end; CI runners on the others follow the site's install instructions word
  for word and run the CLI parts of `test` and `submit`. Each OS gets its own `os` entry.
- **Editor:** Neovim, set up exactly as the site documents, plus a plain Vim setup.
  VS Code is checked as well when there is time.
- **Allowed sources:** https://cp-lang.org and every page it links, the installer it
  points to, the judge's problem pages, and general knowledge of C++ and competitive
  programming. Asking a person, reading the repo, or reading these protocol notes is
  not allowed. Using a source outside the list is a finding (`workaround`).
- **Persona:** fluent in C++, solves Div2 A-D reliably, and impatient: stuck on the
  same thing for 10 minutes, it records a `blocker` and takes the cheapest way around.

## The seven steps
Steps follow MILESTONE-1.md section 5.1 (journeys from EXPERIENCE.md in brackets).

| Step | Task | Passes when |
|---|---|---|
| `understand` (J1) | Open the site; say in 3 sentences what CPL is for and what it is better at; read the intro guide. | The summary matches VISION.md V1-V3, from the site alone, in under 30 minutes. |
| `try` (J2) | In the browser playground, run and change an example, then solve 3 intro exercises. | 3/3 exercises give correct output in the browser, nothing installed. |
| `install` (J3) | Install the CLI and the Neovim setup from the site's instructions. | On every OS `cpl --version` works. In Neovim a new `.cp` file shows diagnostics, completion, formatting and go-to-definition after the one documented setup step, and Vim gets highlighting. |
| `solve` (J4, J5) | Solve the 10 held-out Div2 A-E problems (`problems/div2.txt`), reading statements on the judge. | At least 8/10 accepted by the local judge: the reference C++ and checker agree on the samples plus 200 generated cases. |
| `test` (J6, J7) | For each problem, run the samples and a custom test. For one problem, find a planted bug with the documented stress or debug workflow. | Samples and custom tests ran with the CLI; the planted bug was found from the docs. |
| `submit` (J9) | `cpl build --judge codeforces` for every solved problem. | Every build compiles with `g++ -std=c++17 -O2` and is under 64 KB. There are no real judge submissions: the local judge stands in for Codeforces. |
| `compete` (J5-J9) | A timed virtual Codeforces round: 5 unseen problems (`problems/contest.txt`), 2 hours, no pauses. | At least 4/5 solved in time, and no problem lost to missing ordinary functionality or a tool failure. |

## What the agent records
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

- `steps` has all seven keys, each with `pass` and `minutes` (wall clock).
- `kind`: `blocker` (the step could not pass without it), `workaround` (it passed
  another way, such as a source outside the list or rewriting around a gap), or
  `hesitation` (unsure for a minute or more about how CPL says something).
- `lane` is a folder under `plan/domains/`. `id` is a stable slug, so the same
  finding in later runs merges into one row; reuse ids from `BLOCKERS.md`.
- Every finding needs evidence a lane can reproduce without the transcript.

## After a run
1. Add the JSON, run `research/acceptance/blockers.py`, and commit both with the
   run's transcript summary in `runs/<date>-<n>.md`.
2. Append each lane's new or changed rows to its `worklog.md` as a request.
3. Record the headline (steps passed, minutes per step) in the idioms `metrics.toml`.

Cadence: a run every 2 hours until the milestone passes (due about 09:30 PST
2026-11-23), each followed by the step score and the ranked list in the idioms worklog.
