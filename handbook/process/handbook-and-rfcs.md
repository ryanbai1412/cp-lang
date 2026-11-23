# The handbook and RFCs

| | |
|---|---|
| Kind | Process |
| Owner | Coordinator |
| Status | Active |
| Applies to | Every person and agent working on CP Lang |

## Summary

CP Lang is built by about a dozen agents working in parallel. Each owns one
area of the project, called a *domain* (compiler, standard library, CI and
releases, and so on). Ryan is the human owner of the whole project.

Everything we share about *how we work* lives in two places:

- **The handbook** (`handbook/`) describes how things are **now**: our
  principles, processes, guidelines, checklists, designs and domains. Every
  handbook document has one owner, who keeps it true and edits it directly,
  the same way they would edit code.
- **RFCs** (`handbook/rfcs/`) are short proposals for **major** changes to
  the handbook or to architecture shared by several domains. An RFC is
  decided within 15 minutes, applied to the handbook, and then kept as the
  record of why the change was made.

Most changes are ordinary handbook edits. RFCs are rare.

## Why this exists

Agents working in parallel make three mistakes that cost the project time:

1. **Rules live in someone's conversation instead of the repository.** A
   decision made in one conversation never reaches the other agents, so it
   is made again, made differently, or broken.
2. **Documents drift from reality.** A design note still describes last
   week's plan, and the next reader trusts it.
3. **Documents only make sense to their writer.** They refer to `the earlier
   fix` or `as discussed`, use undefined names, and assume the reader knows
   the writer's history.

Fixing those must not create a fourth problem: process that slows everyone
down.

## Goals

- **One place for every shared rule.** If it isn't in the handbook, it isn't
  a rule. If it is, everyone follows it.
- **Written for an outsider.** The reader is a competent engineer who has
  never seen this project.
- **Documents stay true.** A handbook document that no longer matches the
  code is a bug, fixed as urgently as a failing test.
- **Nobody waits.** Every change has one owner who decides. Nothing waits on
  Ryan; he reviews major changes afterwards and can reverse them.
- **Process stays cheap.** Everything in the handbook together, including
  writing and commenting on RFCs, should take under 10% of anyone's time.

## The escape hatch

**Process, including reviewing and commenting on RFCs, should take under
5-10% of your time. If it takes more, or you are fighting a process instead
of doing the work, skip it and unblock yourself.** This is always allowed,
for every process in the handbook, including tests, gates and this document.
A skipped test or gate is picked up in the hourly post-land review and fixed
forward. Add a line to your commit
message:

```
Skipped: <which process>, <one-line reason>
```

Skips are useful information, not offences. The tech-debt domain reviews them
regularly, and a process that keeps getting skipped is fixed or deleted.

## Editing the handbook

Any agent may fix a handbook document that is wrong, unclear or out of date.
The owner keeps it coherent. For example, if `handbook/ci.md` names a script
that was renamed, fix it in the same commit as the rename.

Record meaningful changes in the document's **Changes** list at the bottom:
one line, with the date and what changed. Git holds the rest.

## When to write an RFC

Write an RFC only when a change meets at least one of these tests:

- **It affects several domains.** Example: changing how commits land on main.
- **It is costly to reverse.** Example: a naming convention that renames
  many standard-library functions.
- **It rewrites a large part of the handbook.** Example: adding, splitting
  or merging domains.

Everything else is a direct edit. **When unsure, edit directly.** Review
catches mistakes, and most changes are cheap to undo.

| Handbook document | Owner | Ordinary edit | Needs an RFC |
|---|---|---|---|
| Quality toolkit and registers | tech-debt | add a mechanism to a register | change the toolkit or how registers work |
| How the factory works | coordinator | fix a stale command | change how landing or lanes work |
| Core design docs | compiler, tech-debt | document a new pass | a new shared interface |
| Standard-library guidelines and checklist | stdlib | add a checklist item | a convention that renames many functions |
| Domains overview | coordinator | clarify wording | add, split or merge domains |
| A domain's charter | that domain | add a test suite or metric | change its mission or scope |
| Single sources of truth | tech-debt | list a new one | change which component owns a fact |
| File structure and toolchain | infra | document a new script | move top-level directories or change build tools |
| Release | infra | add a smoke check | add a channel or change versioning |
| CI | infra | add a job | change what blocks shipping |
| Milestones | coordinator | update status | change the bar |
| This document | coordinator | clarify wording | change the process |

## How an RFC works

1. **Draft.** Copy the template below into `handbook/rfcs/NNNN-short-name.md`
   (the next free number). Name one owner. One page is usually enough.
2. **Ask the affected domains.** Message only the domains the change touches.
   A comment says what would break or what would be better, and takes a few
   minutes to write. No reply means no objection.
3. **Decide within 15 minutes of wall-clock time.** The owner reads the
   comments, decides, and sets the status line to *Accepted*, *Rejected* or
   *Withdrawn*, with one sentence why. Anyone who wasn't available can raise
   it later.
4. **Apply it.** Rewrite the affected handbook documents so they read as if
   the new way had always been the way: merge it into the existing structure,
   remove what it replaces, and update every document it touches. Don't paste
   the RFC in, and don't add "as of RFC 12" notes; a reader of the handbook
   should never need the RFC to understand today's rule. Do this in the same
   or the next commit. The RFC then stays as it is, as the record of why.
5. **Flag it for Ryan.** If the change is major (it touches principles, the
   language's character, or overhauls a process), mark it `Ryan to review`.
   Ryan reads these afterwards and can reverse them. Nothing waits for him.

To change an accepted RFC's outcome later, edit the handbook directly, or
write a new RFC if that change is major too.

## Writing for an outsider

This applies to every handbook document and every RFC.

- Start with what the reader needs to act on, then give the reasons.
- Define every project term the first time it appears, or link to
  `spec/GLOSSARY.md`.
- Give a concrete example before the general rule.
- Use real names: file paths, commands, function names.
- Leave out history the reader doesn't need: no commit hashes, no `as
  discussed`, no `the earlier fix`, no references to conversations.
- Prefer short sentences and plain words. Use a table to compare options
  across several criteria.
- Make it as long as it needs to be, and no longer.

## RFC template

```markdown
# RFC NNNN: <title>

| | |
|---|---|
| Owner | <one domain or person> |
| Status | Draft / Accepted / Rejected / Withdrawn: <one sentence why> |
| Ryan to review | yes / no |
| Changes | <handbook documents this edits> |

## Summary
One paragraph a newcomer can understand.

## Problem
What goes wrong today, with a concrete example.

## Proposal
What changes. Examples first.

## Cost
Time this adds for anyone, and how we will notice if it is too much.

## Alternatives
What else was considered, and why not.

## Prior art
How other languages or engineering organizations handle this.
```

## RFC checklist

- [ ] An outsider can follow the Summary and Problem without reading anything else.
- [ ] Every project term is defined or linked.
- [ ] There is at least one concrete example.
- [ ] The Cost section says how much time the change adds and how we would notice.
- [ ] It names exactly one owner.
- [ ] It lists the handbook documents it will change.

## How we know this is working

- **Process time:** the share of agent time spent on handbook process,
  including RFCs. Target: under 10%. Above that, we cut process.
- **Decision time:** from RFC draft to decision. Target: under an hour, with
  at most 15 minutes of discussion.
- **Skips:** `Skipped:` lines per process. A rising count means a process
  needs fixing.
- **Drift:** handbook documents naming files or commands that no longer
  exist. Target: 0.
- **Misreads:** when a newly started agent misreads a handbook document, the
  document is at fault. Fix it.

## Prior art

| Project | What it does | What we take |
|---|---|---|
| Python PEPs | PEP 1 defines the process. Process and Informational PEPs can be "Active", maintained as living documents. | Living process documents |
| Rust RFCs | A fixed template and a final comment period. Accepted RFCs freeze and drift; the Reference and the Book are maintained separately. | The template, and the split between living docs and records |
| Swift Evolution | One review manager runs each review and closes it. | One owner who closes |
| Kubernetes KEPs | Proposals carry owners, a test plan and graduation criteria. | Measures inside the document |
| Architecture decision records | Short and permanent; superseded, not edited. | RFCs as records |

## Changes

- 2026-11-23: first draft; split into the living handbook and RFCs, nothing waits on Ryan.
