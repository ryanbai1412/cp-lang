# How a change flows

| | |
|---|---|
| Kind | Process |
| Owner | Coordinator |

Every improvement to CP Lang follows the same path from a problem to a
landed, verified fix. A one-line bug fix passes through most steps in a
minute; a language change takes each one seriously. Skipping a step you
don't need is normal.

## The steps

1. **Observe.** Start from something concrete: a program that fails, a
   user report, an acceptance-run blocker, a red metric, a `missing` row in
   a quality register. Write down the smallest example.
2. **Find the right layer.** Is the problem in the language design, the
   spec, the standard library, the compiler, a tool, the docs, or the test
   suite that should have caught it? The layer decides the owner.
3. **Find the general fix.** Ask what class of problem this example belongs
   to, and fix the class. A helper with one use, or a special case in the
   compiler, means the general mechanism is missing. If the general fix is
   too big for now, make the local fix and record the general one in the
   owning domain's plan.
4. **Decide.** Within your domain, decide and log it. Across domains, or
   for something costly to reverse, follow `ownership.md` and, if needed,
   write an RFC.
5. **Update the design and the contract.** For a language change, edit the
   handbook's language document and the spec chapter together, and add or
   change conformance cases. For a std change, the signature, its docs and
   its examples change together. For a shared interface, the design document
   and the contract tests.
6. **Implement, with a test.** The example from step 1 becomes a regression
   test. A feature works in every tool it touches: errors, interpreter and
   trace, C++ output, fmt, lint, LSP, docs. Never weaken a test to land.
7. **Ask what should have caught it.** If the problem reached a user, find
   the quality-register row whose check should have caught it, and
   strengthen that check (`quality.md`).
8. **Land and tell.** Land with `scripts/land.sh`. Fix any handbook
   document the change made wrong in the same commit. If another domain is
   affected, tell its owner in their worklog.

## Example

A user finds that `xs.max()` returns `T` on some sequences and `?T` on
others.

- **Layer:** the standard library's API, so stdlib owns it.
- **General fix:** every search and extremum returns `?T`. That's a naming
  and return convention in `handbook/guidelines/std-api.md`, enforced by the
  API-consistency lint, not a fix to `max` alone.
- **Contract:** the convention is written down, the signatures change, their
  docs and examples change, and a `cpl fix` rule rewrites existing code.
- **What should have caught it:** the API-consistency lint. Its rule set
  gains the return convention.

## Changes

- 2026-11-23: first version.
