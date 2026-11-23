# How the factory runs

| | |
|---|---|
| Kind | Process |
| Owner | Coordinator |
| Describes | `scripts/land.sh`, `plan/STATUS.md`, `plan/domains/*/worklog.md` |

CP Lang is built by a *factory*: about a dozen agents working in parallel,
around the clock, each owning one domain (see `ownership.md`). This document
says how work moves from an idea to a shipped release, and what keeps the
factory fast.

## Goals

- **Ship continuously.** Hundreds of commits land on `main` every hour, and
  a usable edge release comes out at least hourly.
- **Operate in hours, not days.** Plans, checkpoints and estimates are in
  hours. A week of human-team work is roughly one agent-day.
- **Nobody waits.** Every lane always has unblocked work. A lane that is
  blocked unblocks itself: it works around the gap, takes the reasonable
  default and logs it, or fixes the blocker in the other domain and tells its
  owner.
- **Process stays under 10% of anyone's time.** Above that, skip it (see
  `handbook-and-rfcs.md`).

## The roles

- **Lanes.** One agent per domain. It owns its domain's code, its charter in
  `handbook/domains/`, and its plan, metrics and worklog in
  `plan/domains/<domain>/`.
- **Helpers.** A lane may start short-lived helper agents for parallel work
  inside its domain. The lane stays accountable for what they land.
- **The coordinator.** Starts and briefs lanes, reads every lane's reports,
  keeps each one pointed at the current milestone, relays Ryan's decisions
  verbatim, and keeps the handbook's process documents true. It is not a
  domain and owns no code.
- **Ryan.** The human owner. Sets direction and milestones, and reviews
  major changes afterwards. Nothing waits for him.

## The loop

Every stage runs all the time, against the latest `main`.

1. **Find work.** From the current milestone's acceptance scenario
   (`handbook/product/milestones.md`), the domain's quality register (rows
   marked `missing`), its plan, user feedback relayed by the coordinator,
   and requests from other lanes. Blockers of the milestone come first.
2. **Make the change.** Follow `change.md`: find the general fix, update
   the design and spec when the language changes, land code with its test.
3. **Land it.** Commit with a `Domain: <domain>` trailer, then run
   `scripts/land.sh`. It rebases onto `main` and pushes in about 10 seconds;
   the pre-commit hook runs the commit tier (at most 25 seconds). Details in
   `landing-and-ci.md`.
4. **CI and the sheriff.** CI runs the full gate on every `main` tip. Infra
   is the sheriff: it finds the commit that turned `main` red, then fixes it
   or reverts it and tells the owner. Lanes don't stop pushing while `main`
   is red, and no extra pre-push step is added to keep it green.
5. **Release.** The newest commit that passes the gate is published to the
   `edge` channel at most once an hour (`release.md`).
6. **Report.** Each lane appends to its worklog what landed, the evidence,
   what's next and what's blocked. `plan/STATUS.md` is generated from every
   domain's `metrics.toml`.

## Standing reviews

These run on a schedule, without anyone asking:

- **Hourly post-land review** (tech-debt): reads the last hour of diffs for
  architecture problems, duplicated logic, weakened tests and skipped
  safeguards. Findings go to the owning lane and come before its new work.
- **Hourly register audit** (tech-debt): one domain's quality register is
  checked for missing categories and rows without a real check.
- **Ratchets** (`scripts/debt-ratchet.py`, in the gate): counters that may
  only go down, such as tree-walker fallbacks, std names hard-coded in the
  compiler, oversized files and skip lists. Raising one needs a line in
  `decisions.md` in the same commit.
- **Architecture first:** a change that adds a crate, a crate dependency,
  a module or a concept updates the matching design document in
  `handbook/design/` in the same commit. A new crate dependency fails the
  gate until the crate map in `handbook/design/architecture.md` lists it.
- **Change-cost probes** (tech-debt): a fixed set of everyday changes made
  by a fresh agent; files touched and time taken must trend down.
- **Lane review** (coordinator): every lane's latest report is read; a lane
  that has finished its current work gets its next piece, and no lane is
  left idle.
- **Milestone acceptance run** (idioms, every 2 hours): an agent follows the
  acceptance scenario on fresh machines and publishes ranked blockers in
  `plan/product/BLOCKERS.md`.

## Briefing an agent

Give each agent the smallest context that does the job: the goal, the
handbook documents that apply, the tests to pass and the paths it owns. Pass
Ryan's words verbatim rather than paraphrasing them. Long histories poison
judgment, so a fresh agent with a good brief beats a long-running one with a
muddled context.

## Improving the factory

The factory is itself something we build. When agents waste time, the fix
goes into a brief, a script, a check or a handbook document, so that the
next agent starts better than the last. Signs to act on:

- a lane idle or waiting;
- `Skipped:` lines piling up against one process;
- the same mistake made by two lanes;
- landing, CI or release time creeping up.

## Changes

- 2026-11-23: first version, replacing the loop in `plan/OPERATING.md`.
