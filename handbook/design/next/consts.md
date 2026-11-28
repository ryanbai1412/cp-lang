# Constant evaluation as a layer (audit F4)

**Status:** design note, 2026-11-27, lane H, with lane I. Build starts
when `generic(ItemId)` is on the db (the lower move); nothing is built
over `Decls` before then (migration rule 11).

## The problem

`crates/cpl-next/src/consts.rs` is a hidden layer. Its `Evaluator`
holds the whole program (`Decls`, `Loaded`), rewrites a constant into
an old-AST `let` (`cpl_syntax::ast`), and runs lower, own and the
engine from the orchestrator to get its value. The checker asks it
through `check::Consts`. So `cpl-next` holds a layer, constant
evaluation reads the old tree, and every constant re-lowers the program.

## The layer

- A crate, `cpl-next-consts`, after `cpl-next-engine` in the layer
  order: it depends on `cpl-next-db`, `cpl-next-ir`, `cpl-next-lower`,
  `cpl-next-own` and `cpl-next-engine`, and defines its query there,
  as each layer defines its own.
- The query is `const_value(SourceId) -> Result<Option<Scalar>,
  ConstError>`. `SourceId` is item-relative and stable, and inline
  constant expressions (sizes, domain ends) have no item of their own;
  a `const` item is the `SourceId` of its initializer.
- Its compute reads the expression's lowered fn from `generic(ItemId)`,
  and each constant it names as that constant's own `const_value`.
  Then it runs own and the engine's `eval` under `BUDGET` steps.
- A cycle is the db's: `Query::recover` answers `Err(Cycle)`, so the
  evaluator's in-progress set goes. A const cycle is an error whatever
  the order it is met in.
- The checker reaches it only through the `Consts` trait, which the db
  implements over `const_value`. Check never names the crate.

## Counter (hard zero)

`next_hidden_const_evaluators`: each `impl .. Consts for` and each call
of `cpl_next_engine::eval` in a cpl-next* crate other than
`cpl-next-consts` and the engine. It is 2 today (`consts.rs`'s impl
and its `eval`), and reaches 0 in the landing that adds the crate and
deletes `consts.rs` (rule 13).
