# Index models for sequences: options

| | |
|---|---|
| Kind | Options for a decision (Ryan's call) |
| Owner | stdlib |
| Built from | [sequences-usecases.md](sequences-usecases.md) |

## The question

When a contestant asks a sequence "where?" instead of "what?", how do they
say it, and what do they get back? Five use cases depend on the answer:
position search (seq/U2), argmax (seq/U3), sorted with indices (seq/U12),
bsearch as a cut (seq/U4), and keeping positions through filters and views
(seq/U16, seq/U70). Here are three coherent models, each worked through
the same eight moments. `a: [I]Int` throughout, and `s` is `a` sorted.

## The three models

**A. Positions are a sequence too.** `a.keys` is the sequence of `a`'s
positions, so any search over it answers "where?". Values come from
searching `a`, positions from searching `a.keys`, and pairs from searching
`a.items`. There are no position-only names, except `argmax` and `argmin`
as short forms. A bsearch answers with a `Cut`, a point between positions.

**B. Pairs everywhere.** "Where" questions go through `a.items`, so every
answer is `(i, x)`, and you take `.0` when you only want the position.
There's one search vocabulary, and position-and-value is the default.

**C. Named position functions (C++/Rust).** Each search has a position
twin: `position`, `rposition`, `argmax`, `argsort`, `lower_bound`,
`positions`. Familiar, and each name is easy to find in the docs.

## The eight moments

| Moment | A. keys | B. items | C. named |
|---|---|---|---|
| first match | `a.keys.find(i => a[i] == x)` | `a.items.find(p => p.1 == x)` | `a.position(v => v == x)` |
| last match | `a.keys.rev.find(i => a[i] == x)` | `a.items.rev.find(p => p.1 == x)` | `a.rposition(v => v == x)` |
| argmax, rightmost | `a.argmax(ties: Ties.Last)` | `a.items.max(by: p => p.1, ties: Ties.Last)` | `a.argmax(ties: Ties.Last)` |
| sort, keep indices | `a.keys.sorted(by: i => a[i])` | `a.items.sorted(by: p => p.1)` | `a.argsort()` |
| lower bound as a cut | `s.partition_point(v => v >= x)` | same as A | `s.lower_bound(x)` |
| filter, then index | `a.keys.filter(i => a[i] % 2 == 0)` | `a.items.filter(p => p.1 % 2 == 0)` | `a.positions(v => v % 2 == 0)` |
| window's start | `a.windows(2)`, then `w.keys.first!` | same as A | `a.windows(2).enumerate()` |
| items after a filter | `a.items.filter(p => p.1 > 4)` | same | `a.enumerate().filter(p => p.1 > 4)` |

Every A and B cell runs on main today (`Ties` is new). In C, `argmax`
exists; the other names don't.

## What each costs

- **A** has one rule ("search what you want back") and no new names. A
  position found in `a` keeps `a`'s index type, so using it on another
  domain's array is a compile error (seq/U70). It costs a repeated `a[i]`
  inside closures, and newcomers need one sentence to learn `keys`.
- **B** keeps the value next to its position, which helps when you need
  both. It costs `.0` and `.1` noise in the common position-only case, and
  a bsearch over pairs can't return a cut.
- **C** is what a C++ or Rust contestant reaches for. It costs doubling the
  vocabulary: every search, max, sort and filter needs a twin with its own
  tie, empty-input and domain rules, and the twins drift apart. Index
  safety holds only if each twin returns the domain type. `enumerate`
  gives an `Int`, not an `I`, which is the wrong-index-space P0 class.

## Recommendation

**A, with C's familiar names as lint hints, not APIs.** A is the only model
where positions keep their domain for free and every new search works as a
position search the day it lands. Writing `a.position(...)` or
`a.argsort()` should get a lint that names the A spelling, so a newcomer's
first guess is one step from right. Keep `argmax`/`argmin` (four ports use
them, and `ties:` now matches `max`). Use `items` (B) when both position
and value are wanted.

One finding from writing this: our `partition_point(ok)` wants `ok` false
then true, but C++'s `std::partition_point` wants true then false. Under
`cpl run`, the C++ direction (`s.partition_point(v => v < 5)`) returns a
wrong cut with no trap. Whatever model wins, the cut-returning search
should say its direction in its name, as `first_true` does, and
`partition_point` should either match C++ or go.
