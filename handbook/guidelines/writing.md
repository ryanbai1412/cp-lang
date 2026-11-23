# Writing handbook documents

| | |
|---|---|
| Kind | Guideline |
| Owner | Coordinator |
| Applies to | Every document in `handbook/`, and RFCs |

A handbook document is read by someone who has never seen the project: a
new contributor, a friend trying the language, an agent starting cold.
They should understand the point of the document in the first minute, and
only read further if they need to. Write for that reader.

## Shape: overview first, reference last

Every document has two parts.

**The overview** is prose a person reads top to bottom, in a few minutes:

1. **What this is,** in two or three plain sentences, before any heading.
   Say who should care and why.
2. **What good looks like,** as a concrete picture or example, not a
   list of properties.
3. **How it works,** at the level of the main parts and how they connect.
   Short paragraphs or a short list; no tables.
4. **How we know it works,** in a paragraph: the main checks, and the
   weakest areas today.
5. **Where to start:** the two or three common tasks and the first file to
   open for each.

**The reference** comes after a `# Reference` heading: exact scopes,
interface lists, quality registers, command tables, checklists. Tables
belong here, where readers look things up rather than read.

A short process or guideline document may need only a summary, the rules
with their reasons, and a checklist. Keep the order: why first, details
last.

## Guidelines and checklists

Rules come in pairs of documents, so that people can understand them and
machines can enforce them:

- **The guideline** (`handbook/guidelines/<topic>.md`) explains the rules
  at a high level, in prose: what we are trying to achieve, the few ideas
  behind the rules, and examples of good and bad. Read it once to learn how
  to think about the topic.
- **The checklist** (`handbook/guidelines/<topic>-checklist.md`) spells
  out every rule, exhaustively and systematically. Each rule has a stable
  identifier such as `std-api/R12`, one sentence stating it, its reason in a
  line, a short example, and what enforces it: a lint, a test, a gate, or
  `review` where nothing mechanical can decide.

A lint or test that enforces a rule cites its identifier, and a domain's
quality register points to checklist rules instead of restating them. A
rule that exists only in code, or only in a register, belongs in a
checklist. When two rules say nearly the same thing, merge them into one
more general rule.

## Style

- **Define every project term** the first time it appears, or link to
  `handbook/glossary.md`.
- **Show an example** before stating the abstraction it illustrates.
- **Prefer prose to tables** in the overview. A table whose cells are
  sentences is a list of paragraphs in disguise.
- **Keep sentences short and concrete.** Name the file, the command or the
  type; avoid words like "robust", "seamless" or "leverage".
- **No shared context.** No `as discussed`, no session links, no commit
  hashes, no `the new X`. `scripts/check-handbook.py` flags these.
- **Current facts only.** Numbers that change go in `plan/`, not here.

## Checklist

- [ ] A newcomer gets the point from the first paragraph.
- [ ] The overview reads top to bottom without a table.
- [ ] Every term is defined or linked.
- [ ] There is at least one concrete example.
- [ ] Reference material sits under `# Reference`.
- [ ] `scripts/check-handbook.py` passes.

## Changes

- 2026-11-23: first version.
