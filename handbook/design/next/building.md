# Building the new pipeline

Read this before every change to `crates/cpl-next*`, `std/` or the
spec. Each rule names the shortcut it prevents. Every shortcut it lists
was found in the new pipeline and became cleanup work (shortcut audit,
2026-11-27). The design is in `handbook/design/architecture.md` and
this directory; this page is how to build it.

Ryan: "it's actually very important to avoid taking shortcuts ... I'm
just very scared that after we do this migration, we'll realize that we
still took shortcuts and so we still get cooked."

## Rules

1. **Build the designed thing, never a stand-in.** If the component you
   need doesn't exist yet, build it first or ask its lane for it. Don't
   write a consumer against the old syntax tree, a whole-program pass,
   a name match or a constant. The full version costs about the same as
   the shortcut, and the shortcut also costs its cleanup later
   (migration rule 11).
2. **One answer per question.** Representation, layout, ownership,
   operation semantics, name resolution, limits and the facade each
   have one home, and everything else reads it. Before you write a
   predicate, a table or a `match` over operations, find the existing
   one. If it's wrong, fix it there.
3. **Ids after resolution, never text.** After names resolve, there is
   no `String` key and no comparison against a fixed name. std
   behaviour comes from the declared roles (`spec/` role table), never
   from a module, type or decorator name.
4. **Per item by default.** Every query is keyed by an item or
   instance id and reads only what it names. A function that takes the
   whole program needs a line in `incremental.md` saying why.
5. **Each layer decides only its own things.** Check types. Own places
   ownership. Link picks instances. Backends execute what the IR says
   and infer nothing. The checker never decides a lend or drop, and
   link and own never reject a program the checker accepted.
6. **"Not built yet" is not the user's error.** An unbuilt feature is
   an internal `Unbuilt` refusal. It is never a user `E` code, and
   never accepted and then ignored.
7. **A setting either works or is rejected.** An option, decorator
   field or `@checks` level that changes nothing is a silent bug.
8. **Inputs come from the snapshot.** Limits, profile, judge, run
   input and entry points arrive through the snapshot and facade, not
   as constants, environment reads or extra function arguments.
9. **No case-shaped code.** Fix the general rule. If one case needs
   its own branch, the design is missing something: change the contract
   instead.
10. **Tests tell the truth.** A wrong verdict stays wrong. Lists come
    from runs, never from hand edits. A harness never moves a failure
    into a softer bucket.
11. **Debt is counted structurally, from the first line** (migration
    rule 12). A new kind of shortcut ships with its hard-zero counter in
    `scripts/debt-ratchet.py`. Counters match structure, not names, so
    a rename can't evade them, and they cover every `cpl-next*` crate.
12. **Reuse carries its deletion** (rule 13), and **what you replace is
    deleted in the same landing**.
13. **No "for now".** Code isn't written to be replaced. If you can't
    build the full version, refuse with `Unbuilt` and say what's
    missing.

## Before you push

- Does this consume anything temporary, old or name-matched?
- Did I add a second answer to a question that already has one?
- Any new `String` key, fixed-name comparison, whole-program input or
  backend decision?
- Did any counter rise? Does a new kind of debt have its counter?
- Did I delete what this replaces?
- Focused tests, `uv run --quiet python scripts/debt-ratchet.py
  --check`, then push. Process stays under 5% of your time; lane G
  owns CI and full runs.

## What blocks a push and what doesn't

Velocity wins over CI only where the cost can be paid later at the
same price. A broken case can: lane G's runs on main attribute each
case regression to its commit and the owning lane fixes it in a batch,
so case regressions never block a push. Rework can't: a shortcut, a
second answer or a missing counter costs more every day it stays. So
the debt and structure counters block every push, and a wrong-accept
that comes from a shortcut is fixed at once, not batched.

## When the design doesn't cover it

Make the call within the contracts. If a contract is missing or wrong,
change it in the same landing (`contracts/`, `decisions.md`, a
paragraph is enough) instead of working around it. Ask the coordinator
only for a call that crosses lanes.

## Who owns what

| Lane | Owns |
|---|---|
| A | std roles, decorators, intrinsic table, core privilege |
| B | every std module checking as an ordinary library |
| C | name resolution, ids instead of strings, bounds |
| D | link and own errors, ownership with F |
| E | facade, run API, shipped `cpl` switch |
| F | IR ops as one table, representation and layout, engine, ownership |
| G | CI, fast tests, lists from runs, END-STATE measurement |
| H | core expressions, `@checks`, engine events with statement ids |
| I | query database, per-item checker, link inputs |
| J | lossless syntax tree and parser, moving readers onto it |

The switch lands only when every gated `next_*` counter is 0 (migration
rule 14).
