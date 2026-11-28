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
query   items(FileId, module)       -> ItemList                names, kinds, ItemIds, no bodies
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
  statement block), plus the file that holds the module. It never
  depends on byte offsets or on the order in which modules were
  loaded, so the same program gives the same IDs. The file is part of
  its identity (equality and hash), so two programs' `main`s, or two
  `cp.toml`s' `kit.m`, are different items in one session's database;
  what a person reads (diagnostics, `m.name` paths, dump-ir) is
  `module.name`, never the file (lanes I and J, 2026-11-27).
- `items(FileId, module)` reads a file as a module, so a file one
  program opens as its root and another uses as a library is two
  modules. A library module's file is `module_file(root, module)`,
  over the `Libraries(root)` input a session reads from `cp.toml`.
  A query never writes an input: the loader returns the library files
  its database lacks, the session reads them and asks again, a bound
  of one round per file, and a file still missing is an unknown
  module.
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

## Per-module interface store

A module's interface is its `signature`s, its `scope` and its fns'
`summary`s: everything another module's queries read of it through the
four views. Once the views land, interfaces are stored, keyed by
`(ModId, content hash)`. The hash is over the module's text and the
hashes of the interfaces of the modules it imports. Std is checked
once per compiler build, and every program and snapshot shares its
interfaces. A program over std never checks std again: its queries read
std's signatures, scope and summaries from the store, and they reach a
std `body` only to make an instance of it.

Invalidation: a stored interface is valid while its key matches. An edit
to a module changes its hash, so its interface is made again. If the new
interface equals the old one (early cutoff), its importers keep theirs.
If it differs, each importer's hash changes and the importer is made
again, and so on outward. Bodies are not part of the key. An edit inside
a body that leaves the module's signatures, scope and summaries equal
reuses the stored interface, and only that body is checked again. A
compiler build changes every key, because std's key includes the build's
id, so a new compiler never reads an old compiler's interfaces.

Order: after step 1 (the views), before `body(ItemId)` caches across
programs. The test checks two programs over std in one database and
counts zero std `body` or `signature` recomputations for the second.

## Std's answers on disk

A run starts in under 300 ms at p95 (END-STATE). Checking all of std on
every start takes 1.15 s on `print(1)` (lane F, 2026-11-27), so std's
answers are kept on disk and read instead of computed. The store is part
of the database, not a CLI-side std blob, so the CLI, the language
server and the judge read the same one (coordinator, 2026-11-27).

Order:

1. Std is db queries: `Source(std file)` inputs, then `decls`,
   `signature(ItemId)`, `body(ItemId)` and `generic(ItemId)` (the joint
   decls and check landing). `print(1)` then checks only the std bodies
   lower reaches.
2. Measure start after step 1 alone and write the number here, so the
   store's share is known.
3. The store, if step 2 is still over 300 ms.

The store:

- A query opts in with `Persist: Query`, whose key and value encode and
  decode. `Db::open_store(path)` attaches one file.
- The file is `<cache dir>/cpl/std-<build id>-<std hash>.bin`; the
  cache dir is `$CPL_CACHE` or the platform's. The build id is the
  compiler's commit and a hash of uncommitted changes, and the std hash
  is over std's file texts. A new compiler or another std is another
  file, so a stale file is never read. Writing keeps the newest 4 files.
- An entry is `(query name, key) -> (value, deps)`. Its deps are the
  inputs it read, with each one's content hash, and the queries it read,
  with each answer's hash. An entry seeds the database's memo as
  verified only while every input's hash matches the current input;
  otherwise it is computed, with early cutoff, so editing one file of a
  `--std` directory reuses the rest.
- A file whose magic, format version or checksum is wrong, or that does
  not decode, is discarded and its answers computed. The store never
  causes an error.
- The file is a header (magic, format version, build id, std hash), an
  index from `(query, key hash)` to offset, then the entries. An entry
  is decoded when a query first asks for it. It is read whole; `mmap`
  comes only if a measurement says reading is the cost, through
  `memmap2`. A write goes to a temporary file renamed into place, so
  runs at the same time are safe.
- Encoding is `serde` and `postcard`. It is deterministic: maps are
  encoded sorted, never in `HashMap` order, so two runs write the same
  bytes, and the determinism test covers the file.
- A persisted value holds only stable ids: `ItemId`, item-relative
  spans, and `Sym`s resolved through the interner persisted with them;
  never an arena index, a pointer or an `Rc`'s identity. A test fails if
  a `Persist` value type holds a process-local handle.

Tests, in the commit tier: a session in a fresh process over a written
store computes no std `decls`, `signature` or `body` (by
`Db::computed`); a corrupted file gives the same answers as no file.

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

## After link, per instance

Link stays one step per build, but it only joins answers kept per
instance. An instance's key is its `InstKey` (contract 5) and the
hash of its canonical IR. Every query below is cached under that key,
so one edited body redoes only its own instances and the ones that
took something from them.

| Query | Reads | A change to it comes from |
|---|---|---|
| `owned(instance)` | the instance's raw IR, its callees' `Summaries` | its own body, or a callee's summary |
| `optimized(instance)` | its canonical IR, and for each callee it inlined that callee's key and hash | its own body, or a body it inlined |
| `bytecode(instance)` | `optimized(instance)` | the same as `optimized` |
| `cpp_fn(instance)` | `optimized(instance)` | the same as `optimized` |
| `program(Job)` | every instance's key, `owned` and `optimized` | any instance's key changing, or one added or gone |
| `cpp(Job, Profile)` | every `cpp_fn` in instance order | any `cpp_fn` changing |

- The optimizer records each callee it inlined, with that callee's
  key and hash, the way rustc keeps a per-unit inlining record. The
  record is part of `optimized`'s answer, and a hash that differs from
  the callee's current one invalidates it. Today the optimizer works
  within one fn, so the record is empty and the key is the instance's
  own hash. The first inlining pass writes the record in the same
  commit.
- `owned(instance)` uses the same key. Ownership reads callees only
  through their summaries, so a callee whose body changed but whose
  summary did not leaves it valid.
- The engine compiles one bytecode unit per fn and calls fns through a
  function table indexed by instance. A changed fn's unit is swapped
  into its slot and no other unit is compiled again.
- C++ emission keeps each fn's text under the same key and joins them
  into one file, as judges take one file. g++ builds that one file.
- Link keeps its job key. It reads every instance's key and cached
  answers and recomputes none of them.

Test plan, part of the count gate:

- Edit one body that nothing inlines. Exactly its instances get new
  `owned`, `optimized`, `bytecode` and `cpp_fn` answers. Every other
  instance's answers are the same values, by identity, as before.
- Edit a body that another fn inlined. The edited instance and each
  instance whose record names it are redone, and nothing else.
- Edit a body without changing its summary. Its callers' `owned` is
  kept.
- After each edit, the engine's function table has new units only in
  the redone slots, and the joined C++ differs from the previous one
  only in the redone fns' text.

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

What `Whole` holds moves to items, and lowered IR carries no front-end
table beside its fns. Link reads each fact by id from a per-item query:

| Handoff field | Becomes | Keyed by |
|---|---|---|
| `Whole.domains` | `domain(ItemId)`: a domain's name, size and start slots, owner, `counted` | the item that declares it and which of its domains it is (`Decls::dom_ids`) |
| `Whole.globals`, `Generic.globals` | `global(ItemId)`: a top-level binding's name and type, lent mark included | its top-level item |
| `Whole.program_fns` | a fact of `signature(fn)`: the module it is in | `FnId` |
| `Whole.ctx_impls` | a fact of `body(fn)`: the context copy each impl fn's call runs | the impl fn's `ItemId` and the copy |
| `Generic.adds` | `summary(fn)`, which `Summaries` already reads | `FnId` |
| `Generic.deferred`, `Generic.untyped` | `diagnostics(ItemId)`: what checking the body found, said when an instance of it is made | the fn's `ItemId` |

The checker's side tables go the same way: what a body's own nodes need
moves into its typed tree, and what other items read is one of these
queries. `next_handoff_side_fields` counts the handoff fields still
holding a table and shrinks to 0. A field is lowering's own answer only
if it is its fns (`funcs`) or the messages lowering itself makes
(`errors`).

`link` and `own::link` stay per build, keyed by the job, because
closing sets and making instances need every reached fn. `link` takes
only `Inputs` (the `Signatures`, `Generics`, `Summaries` and `Checked`
views) and `Roots`, the ids of the build's fns and globals. Each view
answers one item by id and none lists a whole table, so the database
serves it unchanged; until then `Decls`, the lowered map, `SetAdds` and
`Answers` implement them. `link_reads_check` follows the data: a line of
cpl-next-link holding `Whole`, `Deferred`, `SetAdds` or a whole-program
value, directly or through a holder struct, counts. `own::link` stays on
`next_whole_program_inputs` until it takes the views too. At that point an edit to one body redoes only that body's
`body`, `generic` and `lent`, its instances and their `owned`, and then
link. The count gate checks this.

Order:
1. The views and their map implementations in `cpl-next-decls`. Then
   `cycles`, `plain`, `fresh_dom`, `supertraits_unmet` and `body_type`
   read them. That takes the counter from 13 to 6.
2. `body(ItemId)` (step 3 below).
3. `generic(ItemId)` and `lent(ItemId)`. `Whole` goes: `domain`,
   `global` and `diagnostics` replace its fields one by one, each
   landing with `next_handoff_side_fields` lowered.
4. `owned(instance)`, and link reading the views.

Each step lands with `next_whole_program_inputs` lowered in the same
commit.

### `body(ItemId)`: what one body reads and answers

A body is checked in its own type and set variables (`Cx::enter_store`),
after the top level has written out its bindings' types. What it reads
from outside itself is one of these answers, never another body's
variables:

| Reads | Answer |
|---|---|
| a fn's signature, a type, a trait or impl | `Signatures`, `TypeTables`, `Resolve`, `Roles` |
| what a name means in its module | `Scope`, by `Sym` |
| a top-level binding's type, a top domain | `global(ItemId)`, `domain(ItemId)`: the top level's own answers |
| a callee's `_` result | `ret(FnId)`: the callee's `body` read for its result; a cycle reads the result as declared |
| a library `const`'s value and type | `const_value(SourceId)` (consts.md), keyed by the const, not by which body met it first |

What it answers, each keyed by the fn and its context copy, so no
answer depends on the order bodies are checked in: its typed tree and
locals; the lambdas, domains, ranges and set calls it makes, by their
place among the copy's (`Ids`); what it adds to its own set parameters
(`summary`); what it puts in a global's set; the context copies and the
fns it reaches; and its diagnostics (`diagnostics(ItemId)`). A library
`const` becomes its own item with its own global, not a global
appended when a body first reads it.

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

6. Diagnostics are a query (trend audit #15, item 3), after `body(ItemId)`
   and before link reads per-item answers in step 5. `body(ItemId)`
   answers its own diagnostics and, apart, its deferred ones: those of
   a library fn reached only through a trait, said only if an instance
   runs it. `generic(ItemId)` answers lower's for a quiet fn the same
   way. `program(Job)` answers which fns its instances run, and the
   facade joins each item's diagnostics with the deferred ones of each
   fn that runs. `Deferred`, lower's copy of the checker's, and link's
   `l.errors.extend(c.deferred(g))` are deleted: link says only its
   own. `link_reads_check` (lane G) follows relays and trait objects,
   so a checker diagnostic that reaches link through lower or a `&dyn`
   fails it.

Steps 1 to 3 each land on their own and make the batch pipeline
cleaner even before the database exists.
