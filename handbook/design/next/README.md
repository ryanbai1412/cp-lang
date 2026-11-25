# CP Lang architecture: the doc set

**Status:** started 2026-11-24, from the fourth draft of `design.md`.
Unaudited. `design.md` stays the full argument until every section of
its Part 5 has moved here.

This set replaces one long design document with short ones. The reason
is what three audit rounds found: the layers were right, but each
mechanism was designed alone, and nothing said exactly what one layer
hands the next. A contract doc says that, as data shapes.

## The layers

```
 snapshot ─▶ SYNTAX ─▶ DECLARATIONS ─▶ TYPED TREE ─▶ GENERIC IR ─▶ INSTANCES ─▶ raw ─▶ canonical ─▶ optimized
                │            │              │             │                      │          │            │
                └────────────┴──────────────┴─────────────┴──────── FACADE ──────┤          │            ├─▶ C++ emitter
                                                                                 ▼          ▼            ▼
                                                                         reference     engine       engine (run),
                                                                         interpreter   (debug)      machine code
                                                                                 └────────── EVENTS ─────┘
```

Three rules hold everywhere:

1. Each fact is decided in one layer and read by the rest.
2. Every tool goes through the facade over an immutable snapshot.
3. Every node of every form carries the identity of the source it came
   from.

## Contracts, one per boundary

Each says: the data that crosses, what the producer guarantees, what
the consumer may assume, what the consumer must never do, how the
contract is checked, and what is still open.

| # | Boundary | Doc |
|---|---|---|
| 0 | the world → a snapshot | `contracts/00-snapshot.md` |
| 1 | snapshot → syntax tree | `contracts/01-syntax.md` |
| 2 | syntax → declarations | `contracts/02-declarations.md` |
| 3 | declarations → typed tree | `contracts/03-typed-tree.md` |
| 4 | typed tree → generic IR | `contracts/04-generic-ir.md` |
| 5 | generic IR → instances (raw) | `contracts/05-instances.md` |
| 6 | raw → canonical → optimized | `contracts/06-stages.md` |
| 7 | instance IR → engines and C++ | `contracts/07-execution.md` |
| 8 | execution → events | `contracts/08-events.md` |
| 9 | everything → tools (the facade) | `contracts/09-facade.md` |

The data shapes are written in a Rust-like notation. They state what
must be recorded, not how it is stored.

## Facts, one doc each

A fact doc says where the fact is decided, which contracts carry it,
and who reads it: `facts/identity.md`, `facts/types.md`,
`facts/ranges-and-proofs.md`, `facts/ownership.md`, `facts/effects.md`,
`facts/layout.md`, `facts/costs.md`.

## Supporting docs

- `representation.md`: owned-or-lent modes, witnesses, function-value slots, summaries, and the rule that whole-program steps do not reject (round 5).


`slice.md` (the working prototype and what it found), `decisions.md`,
`migration.md`. Requirements and the audit record are still Parts 2 and 8 of `design.md`.

## From the product docs (2026-11-24)

Requirements from the journeys that needed a data shape, and where each now lives. None is reviewed or built.

| Requirement | Journey | Contract |
|---|---|---|
| Run part of a program | J5 | 0 (`Entry::Call`), 7, 9 |
| Interactive problems and graders | J6 | 0 (`Io::Peer`), 7 |
| Contest folder | J4 | 0 (`Workspace`) |
| One way to show a value | J7 | 8 (`show`), 9 |
| Compare two runs | J7 | 8 (`At`, `align`), 9 |
| Parallel stress and tests | J6, J7 | 7, 9 (`run_many`) |
| Performance view with proven, estimated, unknown | J8 | 9 (`costs`) |
| Each judge's compiler | J9 | 7 |
| Reuse of the back half between edits | J5 | 5, 6 |
| Profiling | J8 | 8 |
| AI assistance | J2, J5 | 9 |

Left out on purpose: `cpl submit`, a package registry, and rebuilding old solutions with old compiler versions. They sit outside the compiler; the snapshot's library pins and compiler version are the hooks.
