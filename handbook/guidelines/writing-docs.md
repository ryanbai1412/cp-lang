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

## Two layers: the picture and the exhaustive reference

Every topic is described twice, for two different jobs:

- **The high-level document** gives the picture: what we are trying to
  achieve, the few ideas behind the details, examples of good and bad, and
  how to decide a case nobody wrote down. People read it once to learn how
  to think about the topic, and use it when making decisions.
- **The exhaustive reference** enumerates every rule or case,
  systematically, each with a stable identifier. Tests, lints and reviews
  cite it, so it is what machines enforce and what settles a precise
  question.

| Topic | High-level document | Exhaustive reference | Enforced by |
|---|---|---|---|
| The language | `handbook/language/` | `spec/`, rules like `R04.4.30` | Conformance cases citing each rule |
| A kind of work | `handbook/guidelines/<topic>.md` | `handbook/guidelines/<topic>-checklist.md`, rules like `std-api/R12` | Lints and tests citing each rule |
| A domain | The overview of `handbook/domains/<domain>.md` | Its quality register, under `# Reference` | The check named in each row |
| A component | `handbook/design/<component>.md` | Its interface contracts | Contract tests |

The two layers change together. A rule added to the reference that the
high-level document can't explain means the picture is missing an idea; a
new idea in the high-level document that changes no rule is not yet real.
When they disagree, that is a bug: the high-level document says what we
meant, and the reference is fixed to match, or the intent is revisited.

In a checklist, each rule has its identifier, one sentence stating it, its
reason in a line, a short example, and what enforces it: a lint, a test, a
gate, or `review` where nothing mechanical can decide. A lint or test that
enforces a rule cites its identifier, and a quality register points to
rules instead of restating them. A rule that exists only in code, or only
in a register, belongs in a checklist. When two rules say nearly the same
thing, merge them into one more general rule.

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
