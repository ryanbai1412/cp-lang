# tech-debt

Segment: B. Metric. Owns: crate/file layout, dead code, duplicated helpers, stdlib privilege, 0.2 leftovers. Worklog: [worklog.md](worklog.md).

## Covers
Anything that makes the code harder to change than it needs to be: giant files, duplicated logic, compiler special cases for the stdlib, dead code, 0.2 leftovers, inconsistent naming and layout, stale docs. Pure refactors, no behavior change, each landing with its ratchet lowered.

## Improve

- Split the 24 `.rs` files over 2,000 lines (largest: `interp/vm.rs` 15.4k, `ir/lower/call.rs` 13.4k, 4 more `ir/lower/*` over 4.5k); `vm.rs` split is ready on a branch.
- Remove stdlib privilege: the 77 library-shaped compiler builtins move to std (with stdlib).
- Delete 0.2: the 94 dead edition refs, `CPL_SPEC`-style mode switches, archived programs classified then removed.
- `spec/std` -> `std/`; `prover` -> `cpl-prover`; one interpreter VM (consolidation in ARCHITECTURE.md).
- The 19 `#[ignore]` tests: fix or delete with a reason.
- Duplicated-helper audit re-run on everything added since the last one.
- Auto-generate the tables in `STATUS.md`/`REPO.md` so docs can't go stale.
- The uncountable part (most of the real debt): wrong or leaky abstractions, special cases where a general mechanism belongs, half-finished migrations, unclear crate boundaries, one concept done two ways, confusing code and stale comments, opaque test infrastructure. Run it with:
  - **Debt register** `plan/domains/tech-debt/register.md`: each item says where, why it hurts, the fix, and impact x effort; worked top-down.
  - **Cold reviews**: a fresh agent reviews one crate/area at a time against a fixed rubric and adds findings to the register (rotating through the codebase).
  - **Change-cost probes**: a fixed set of everyday changes (add a std method, a diagnostic, an LSP feature, a lint) done by a cold agent; record files touched, time and where it got stuck.

## Success criteria
Ratchets (`scripts/debt-ratchet.py`) only go down, to: `rs>2000` 0 (today 24), `rust_ignore` 0 (19), 0.2 refs 0 (94), builtins at an agreed floor (77), `stdlib_todo` 0 (294, with stdlib); every exception logged in `decisions.md` with how to remove it; 0 open high-impact register items; rubric scores per area rising across review rounds; change-cost probes getting cheaper (files touched, time); a weekly debt report in the worklog.

## Ryan's role
Signs off the ratchets and the review rubric; skims the weekly debt report and register top 5.
