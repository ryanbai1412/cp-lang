# The standard library, as a contestant meets it

| | |
|---|---|
| Kind | Product area |
| Owner | stdlib |
| Changes | Edit directly; a changed design gets a use-case entry |

std is everything you call once the input is read. Each surface below is
one kind of thing you reach for mid-contest. Its doc says what using it
should feel like, and its use cases list every need it serves, with the
evidence. The language itself is the language area; this area is the
library.

**Sequences** are what you reach for first: arrays, lists, slices, strings,
ranges and the generators over them. You search, sort, scan, group and
take extremes without a loop, and positions stay in the array's own index
space. [sequences.md](sequences.md), [use cases](sequences-usecases.md).

**Sorted containers** (`SortedSet`, `SortedBag`, `SortedMap`) are the
`std::set` and `multiset` you keep open across a sweep. Insert, erase one
copy, step to a neighbour and count a range, each in one call, with the
neighbour as `?K` and never an end iterator. Not drafted yet.

**Range structures** (`SegTree`, `Fenwick`, sparse tables, `Window`) answer
"fold of this range" and "first position where" while values change. You
name the operation (`+`, `min`, a `Monoid`) once and every query follows.
Not drafted yet.

**Graphs and trees** start from the edge list you just read. You get a BFS,
Dijkstra, LCA or DSU from one constructor over the vertex domain, and a
vertex never mixes up with an edge. Not drafted yet.

**Strings** are sequences of `Char` with the string algorithms added:
hashing, Z, prefix function, suffix array and automata. They take the same
calls as any sequence, plus the ones only text needs. Not drafted yet.

**Math** is `Mod`, combinatorics, number theory, `BigInt`, polynomials and
matrices. A modulus is declared once, and from then on the arithmetic is
right without a `% MOD` per line. Not drafted yet.

**Grids** are 2-D arrays over a row and a column domain. Neighbours, walls,
flood fill and prefix rectangles are free of manual bounds checks. Not
drafted yet.

**I/O** reads the statement's format as types (`read<[N]1..=1e9>()`) and
prints any value. Multitest and interactive problems take the same code as
single-test ones. Not drafted yet.

## Changes

- 2026-11-23: first version; sequences drafted (RFC 0002 pilot).
