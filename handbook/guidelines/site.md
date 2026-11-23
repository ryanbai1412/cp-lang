# Writing and building site pages

| | |
|---|---|
| Kind | Guideline |
| Owner | site |
| Describes | `web/`: Learn, guides, Language chapters, the cheat sheet, `/setup`, `/errors`, `/lints`, the rendered std reference, the playground and embedded editors |

Read this before you add a page, a practice problem, a std reference feature or an editor feature. The exhaustive rules, each with an id that checks cite, are in `handbook/guidelines/site-checklist.md`.

## The idea

A reader trusts the site only if nothing on it is wrong, and uses it in a contest only if it gets out of their way. Almost every rule follows from those two.

**Nothing on the site is wrong.** When `max()` started returning an optional, the build failed on the two guide pages that printed `a.max()`, before any reader saw them. That works because every program on a page is compiled and run in the build that ships it, every fact that more than one page states is read from one file, and the std reference, lint and error pages are generated from the compiler instead of typed. Language chapters explain why a design beat its alternatives and link the spec for the exact rule, so they cannot drift from it.

**The site gets out of the way.** A contestant mid-contest wants one fact: what `by:` takes, what the cost is. So a reference entry is a structure (highlighted signature, a parameter table, labelled cost, empty, order and traps rows, examples that run), not a paragraph. Anything that navigates opens in a new tab with Cmd+click, because the reader keeps their program open. The browser editor is the language server their desktop editor runs, so it answers the same way, and aids like inlay hints can be switched off. Pages fit a phone and load the compiler only when the reader runs or edits code.

## Deciding a case nobody wrote down

Ask what a contestant who copies from this page, or acts on it in a contest, would lose if it were wrong or slow, and make the build catch that. If a rule would need restating on a page, link its source instead. When a new kind of page or editor feature needs a rule, add it to the checklist with an id and point a check at it.

## Changes

- 2026-11-23: first version.
