# The incremental core

**Status:** decided 2026-11-26, not built. It says how the layers are
stitched together so the facade's typing-path guarantees (contract 9)
are true by construction rather than by care.

## Why now

The pipeline today is one batch call (`front` in `crates/cpl-next`):
load every module into one offset space, collect every declaration,
check every body, lower everything, and re-run the whole thing when a
method turns out to need another std module. Each layer takes the
whole program (`check(&Decls, main, &Loaded)`). Nothing can be reused
across an edit, so the editor rechecks the program on every keystroke,
and every layer is free to read anything, which is how facts leak
between layers. Fixing this after the tools are on the facade means
changing every layer's signature twice. Fixing it now changes them once.

## The rule

Every layer is a set of pure functions of explicit, small inputs. A
database calls them, memoizes the result, and recomputes one only when
an input it read has changed. Nothing else in the compiler holds state.

```text
input   file_text(FileId) -> Text, config() -> Config         set by the snapshot
query   syntax(FileId)              -> SyntaxFile              contract 1
query   items(FileId)               -> ItemList                names, kinds, ItemIds, no bodies
query   scope(ModId)                -> Scope                   what each name in a module means
query   signature(ItemId)           -> Signature               contract 2, never reads a body
query   impls(TraitId)              -> ImplIndex               coherence, per trait
query   providers(name)             -> List<ItemId>            which module declares a method; replaces the reload loop
query   body(ItemId)                -> TypedBody + diagnostics contract 3, reads its own syntax and the signatures it names
query   summary(ItemId)             -> Summary                 lends and effects, from the signature only
query   generic(ItemId)             -> GenericFn               contract 4
query   instance(ItemId, TypeArgs)  -> InstanceFn              contract 5
query   program(Job)                -> Program                 own's lend marks, link, own's checks, canonical, optimized
query   cpp(Job, Profile)           -> CppFile
query   spans(ItemId)               -> SpanTable               local node index -> place in the file
```

The facade's queries (`hover`, `diagnostics`, `complete`, ...) are
built from these. `Session::with_file` sets one input and keeps the
database, so a new snapshot shares every answer the edit did not touch.

## Identity, so an edit stays local

- An `ItemId` is a path: module, item name, and a disambiguator for
  impls and repeated names (`impl Show for V`, the k-th top-level
  statement block). It never depends on byte offsets or on the order
  in which modules were loaded, so the same program gives the same IDs.
- A node inside a body is `(ItemId, local index)`. Spans inside a body
  are relative to the item's start. The absolute place is the separate
  `spans` query, which is the only thing a blank line above the item
  invalidates. The single offset space in `modules.rs` goes.
- Diagnostics, events, trace records and IR nodes carry this
  `SourceId`, never a line number.

## Early cutoff

A query whose new answer equals its old one stops the recomputation
there. That is what makes the common edits cheap:

| Edit | Recomputed |
|---|---|
| a character inside one fn body | `syntax` of the file, `items` (equal, stops), that fn's `body`, `generic`, its instances |
| a blank line above a fn | `syntax`, `items` (equal), `spans` of the items below |
| a fn's signature | its `signature`, then the bodies that name it |
| a std module | never, in an editor session: std is a durable input |

Answers are compared by value, so every answer type is `Eq` and holds
no pointers into another answer.

## Std is a library that is read once

Std's `syntax`, `items`, `signature`s and `body`s are queries like the
program's, marked durable: a snapshot that changes only user files
never re-validates them. The precomputed `Std::decls()` and the
"load more modules and start again" loop in `front` are replaced by
`providers`, a query over std's item index. Core gets nothing here that
any other library does not.

## Diagnostics

Each query returns its diagnostics with its answer (contract 9). The
facade collects them from the queries it ran, so a cached body's
diagnostics are still shown, and there is no second pass that filters
diagnostics by span overlap (today's `unread` filter in `front`). A
refusal or an error type is local to the body that produced it.

## Whole-program steps

Ownership's lend marks before link and its checks after (contract 4), linking, canonicalization, optimization and C++
emission need the whole program. They are queries keyed by the `Job`
(contract 9's round 5), and they read per-item answers. They are on
the idle and build paths only. They never report an error the typing
path did not (`representation.md`: whole-program steps do not reject).

## Cancellation and threads

An input change bumps the revision. A query running for an older
revision stops at the next query boundary and its caller gets
`Cancelled`; the newer snapshot never waits. Bodies of different items
are independent, so the database checks them on a thread pool.

## The engine

We use `salsa`, the query engine rust-analyzer is built on, rather
than writing one. It already has early cutoff, durability, cancellation
and parallel queries. Only the crate that wires the layers together
depends on it.

Layer crates do not depend on `salsa`. Each exports plain functions
over small views: `check_body(sigs: &dyn Signatures, item: &ItemSyntax)
-> TypedBody`. `Signatures` is a trait in `cpl-next-decls`; the
database implements it by calling the `signature` query, and a test
implements it with a map. So the layers stay unit-testable, and the
database is the only place that knows what depends on what.

## How it is checked

- **Build graph:** no layer crate depends on `salsa` or on the database
  crate.
- **Count gate** (contract 9's): a fixed list of edits to fixed files,
  each with the number of bodies rechecked, signatures recomputed and
  instances made. The table above is its first rows; a change that
  makes any count grow fails.
- **No whole-program inputs:** a debt counter of public layer functions
  that take a whole-program value (`&Decls`, `&Typed`, `&Loaded`).
  It shrinks to the whole-program steps above.
- **Determinism:** contract 0's check, every query twice on a reloaded
  snapshot with byte-identical answers, covers the IDs.

## The whole-program inputs, one by one

`next_whole_program_inputs` counts the fns of the new pipeline that take
`&Decls`, `&Typed`, `&Whole` or a map of every fn. Each one keys its
answer on the whole program, so any edit redoes it. Each becomes a
query keyed by an item, or a plain fn over a small view. The database
implements the view with queries, and a test implements it with a map.
The views:

- `Signatures`: `signature(ItemId)` for a fn, struct, enum, alias,
  trait or domain. It never reads a body.
- `Scope`: `scope(ModId)`, what a name means in a module.
- `Summaries`: `summary(ItemId)`, what a fn's body adds to its set
  parameters and what it lends. Computed from bodies' recorded calls,
  and no body is checked again.
- `Generics`: `generic(ItemId)`, one fn's generic IR.

Each view is a trait. Once a site's step is done, no layer fn it
became takes `&Decls` or a map of every fn.

| Site today | Becomes | Reads |
|---|---|---|
| `check::check` | `body(ItemId)`: one body, zonked, with its set calls and the ids `prior` gave it; `check` over the program becomes the database's loop | `Signatures`, `Scope`, `Summaries` for a `_` result |
| `cycles::alias_cycle` | a diagnostic of `signature(alias)` | `Signatures` of the aliases it names |
| `cycles::field_holding_itself` | a diagnostic of `signature(struct)` | `Signatures` of the structs its fields name |
| `plain::plain`, `plain::plain_in` | `plain(StructId)`, cached per struct; for any other type it is a match | `Signatures` of fields and variants |
| `collect::fresh_dom` | deleted: a domain's id is interned by its `ItemId`, not counted | none |
| `impls::supertraits_unmet` | a diagnostic of `signature(impl)` | `impls(TraitId)` of each supertrait |
| `tyconv::body_type` | a plain fn over the two views | `Scope`, `Signatures` |
| `lower::lower` | `generic(ItemId)`: lowers one typed body | `body(ItemId)`, `Signatures` |
| `own::lent::mark` | `lent(ItemId)`: one generic fn's lend marks | `generic(ItemId)`, callees' `Signatures` |
| `link::link` | `program(Job)`, a whole-program step (above): it keeps the job as its key but reads only per-item answers | `Generics` after `lent`, `Summaries`, `Signatures` |
| `own::inst::check` | `owned(instance)`: one instance's ownership diagnostics and settled places | that instance, callees' `Summaries` |
| `own::link` | the `program(Job)` query itself: it runs the per-item queries and link | the views above |

What `Whole` holds moves to items. A domain's size and start are its
`signature`. A global is its top-level item's. A deferred diagnostic
belongs to the `body` of the fn it is about.

`link` and `own::link` stay whole-program, keyed by the job, because
closing sets and making instances need every reached fn. They stay on
the counter until they take the views instead of `&Decls` and maps of
every fn. At that point an edit to one body redoes only that body's
`body`, `generic` and `lent`, its instances and their `owned`, and then
link. The count gate checks this.

Order:
1. The views and their map implementations in `cpl-next-decls`. Then
   `cycles`, `plain`, `fresh_dom`, `supertraits_unmet` and `body_type`
   read them. That takes the counter from 13 to 6.
2. `body(ItemId)` (step 3 below).
3. `generic(ItemId)` and `lent(ItemId)`. `Whole` goes.
4. `owned(instance)`, and link reading the views.

Each step lands with `next_whole_program_inputs` lowered in the same
commit.

## Order of work

1. Item paths and item-relative spans, with the `spans` query. This is
   lane I's node-ID work, keyed by `ItemId`.
2. Split declarations into `items`, `scope` and `signature` per item;
   `providers` replaces the reload loop.
3. The checker checks one body against `Signatures`; `check` over the
   whole program becomes a loop the database runs.
4. The database crate, wiring 1 to 3 as queries; the facade's typing
   path reads it. The count gate starts here.
5. `generic`, `instance` and `program` as queries; ownership, on both sides
   of link, reads per-item summaries.

Steps 1 to 3 each land on their own and make the batch pipeline
cleaner even before the database exists.
