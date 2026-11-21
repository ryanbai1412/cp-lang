# Stdlib area designs

One file per area, `design/stdlib/<area>.md`, written by the area's
owner (the table in `plan/STDLIB.md`, "Who owns what"). The file is the
design; the area's section of `plan/STDLIB.md` is the surface it lands
as. The coordinator reviews each design before it lands; a design that
fails a point below is sent back with the point named.

## Review checklist

1. **Raised by ports.** Every new item cites the friction files or ports
   that asked for it, and names the port rewritten to use it as its
   regression test.
2. **One vocabulary.** A name says what it says elsewhere in
   `plan/STDLIB.md` section 2: `add`/`remove`/`toggle`/`in` and the
   `len` getter for collections, `push`/`pop`(`_front`) and
   `first`/`last` for sequences, `get`/`set`/`update`/`fold`/`apply`
   for range structures, `first_ge`/`first_gt`/`last_le`/`last_lt` for
   sorted lookup, `try_<verb>` for a fallible twin, `mark`/`undo_to` for
   rollback, textbook algorithm names. A new word for an existing idea
   is renamed; a reused word with a new meaning gets a new name.
3. **Options are named arguments.** `by:`, `init:`, `cmp:` and friends,
   never a `_by` or `_with` twin. Defaults make the common call short.
4. **Cost on every signature**, in the `///` comment and in
   `plan/STDLIB.md`, written as section 2 says. Anything O(n) or
   allocating looks like it.
5. **No privilege.** Only public language and section 3 intrinsics. A
   missing power is proposed in section 3 for the core lane, not
   special-cased.
6. **Prelude budget.** A new prelude name shows it is used by more than
   one port in ten and says what it displaces; the cap is 80
   (`prelude_stays_under_budget`).
7. **Every tool.** Interpreter and C++ agree (a differential fixture or
   a port gate), docs in `site/content/stdlib.md`, editor tables when a
   prelude name changes, a `decisions.md` entry with its reverse.
8. **Matched speed.** Hot structures are benchmarked against the C++
   they replace (AtCoder Library, KACTL) at the same algorithm.

## Routing frictions

A stdlib friction goes to the area of the type it is about, or the
area that would own the missing item. The coordinator keeps the ones
that change sections 1 to 3 (naming, a new intrinsic, a core feature)
and sends core needs to the core lane.

## Reviews

Designs that landed before this checklist existed are reviewed after
the fact; each numbered item is owed by the area owner, or answered in
the design with a reason.

**`math.md`** (number theory, combinatorics): passes 1, 3, 5, 6.
1. Write expected costs as section 2 says, `// O(x^(1/4) log x)
   expected`, not `expected O(...)` (`primitive_root`, `factor_big`).
2. `Z.comb(limit)` answers the `Comb<P>` friction (14 friction reports); move
   those ports to it in the same commit as the method, with the
   friction files marked fixed.
3. `convolve_mod` and `convolve_int` are separate names because they
   take different element types and run different algorithms; say so in
   one line of the design, so the next area does not read them as a
   licence for type-suffixed twins.

**`strings.md`**: passes 1, 4, 5, 6.
1. One idea, one name: `SuffixArray.lcp_between(i, j)` and
   `RollingHash.lcp(a, b)` are the same question; both are `lcp`.
2. `lcp_with(a, other, b)` is a `_with` twin; make the second structure
   a named argument, `lcp(a, b, of: other)`.
3. `compare` returns an `Int` sign for an order; return the spec's
   `Ordering` (spec 40) once the core has it, and say so as [new, core
   lane] meanwhile.

**`geometry.md`**: passes 1, 4, 5.
1. `Vec.rot_by(u)` is a `_by` twin of `rot(rad)`; name the rotation by
   a unit vector for what it is (`rot(u)` on integer `Vec`, `rot(rad)`
   on `FVec`), or justify both.
2. `locate` returns 1, 0 or -1 for inside, boundary, outside: a
   category, not a sign, so a contestant can test the wrong value.
   Return an enum; keep `turn`'s `Int` sign, which is arithmetic
   (`turn(a, b, c) * turn(a, b, d) < 0`).
3. `convex_hull_idx` follows section 2's index-space rule (a `[D]Pt`
   in, `List<D>` out); keep the name, but every other function with a
   `[D]` twin uses the same `_idx` suffix.

**`seq.md`** (sequences): passes 1, 3, 5.
1. `prod` and the test forms `count(xs, p)`, `all(xs, p)`, `any(xs, p)`,
   `first(xs, p)` landed as intrinsics (decision 26.9); drop
   `pub fn prod` from `std.seq` and list the two-argument forms.
2. `xs.find(pred)` is not a spelling: `find` is the position search on
   lists and the substring search on `Str`; the first element matching
   is `first` (decision 26.9).
3. `.items` as the one `enumerate` is right; make `xs.enumerate()` a
   missing-name hint that names `.items` rather than a second spelling.
4. In `plan/STDLIB.md` costs trail the signature (section 2); the
   comment-above form is fine in the design but not in the surface.
5. `excl_fold` is also `Rooted.excl_fold` in `trees.md`; keep one shape,
   `yields (x, rest)` on a list and `(c, rest)` on a tree, both named
   `rest`, and say in each where the other lives.

**`containers.md`**: passes 1, 2, 4, 5.
1. `MonoDeque.by(better)` makes `by` a constructor; options are named
   arguments, so it is `MonoDeque(by: better)`, with `new(pick)` kept
   only if both read naturally.
2. `PotentialDsu` joins `DsuOf` and `ParityDsu`: same `mark`/`undo_to`,
   `same`, `size`, `components`, `consistent`; good. `ParityDsu` is the
   potential DSU over `Z/2`; say whether it becomes an alias.
3. `offline_over` (time divide and conquer) is owned here, not in
   graphs; the table in `plan/STDLIB.md` says so.

**`flows.md`** (flows/matching): passes 1, 2, 3, 5.
1. Every `CostFlow` accessor gets its own `// O(1)`, as `FlowGraph`'s
   do.
2. `max_closure`, `circulation`, `max_bounded_flow`,
   `min_bounded_flow`: put the cost on each signature, not once below.
3. `reset()` answers `cf2147h`; that port moves to it in the same
   commit.

**`trees.md`**: passes 1, 2, 3, 5.
1. `fold_path`/`apply_path` reuse the range verbs; good. `chains`
   yields `Range<Pre>`, matching 4.10's `Range<D>` arguments.
2. `LinkCut.cut`, `connected` and the rest need their own costs.
3. See `seq.md` 5 on `excl_fold`.

**`ranges.md`** (range structures): passes 1, 3, 4, 5.
1. `Fenwick.lower_bound(target)` is the C++ name; the range searches
   are `max_right`/`min_left` (section 2), so it is
   `max_right(from, ok)` like the others, or justify the difference.
