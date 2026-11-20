# 1.5 State: ownership, borrowing, rollback

Status: the ownership model is decided. The precise borrow rules are
proposed, based on research (research/RECOMMENDATIONS.md). `defer` and
rollback are proposed.

## Ownership

- **Variables own** their values. `let a: List<Int> = read()` owns the list.
- **Parameters borrow.** `fn f(a: List<Int>)` reads the caller's list with no
  copy. `fn f(mut a: List<Int>)` borrows it mutably, and the caller writes
  `f(mut a)`. Method receivers need no marker: `st.update(i, x)`.
- **Big values are copied only explicitly.** Arrays, lists, maps, sets, and
  records holding them need `.copy()`. A value used for the last time is
  moved implicitly, at no cost.
- **There are no pointers and no stored references.** Identity that must
  persist uses typed handles into pools: `Pool<Node>` hands out `Node.Id`
  values, which are 4 bytes.
- **Constructors take arguments by value:** moved on last use, otherwise
  `.copy()`. They have no side effects, and there are no user-defined
  destructors.

```
var g = Dsu(V)
fn link(mut d: Dsu, a: V, b: V): d.union(a, b)
link(mut g, u, v)
let h = g.copy()            // explicit copy of a big value
consume(g)                  // last use: moved
```

## `mut fn`

A nested `mut fn` may mutate `var`s declared above it. It can't be stored or
returned, so its borrow ends at its last call.

```
var dist: [V]?Int = none
mut fn relax(v: V, d: Int): if d < (dist[v] ?? Int.MAX): dist[v] = d
```

## Borrow rules (proposed)

- **Exclusive mutation.** While a place is borrowed mutably, nothing else
  reads or writes it. While it is borrowed for reading, nothing mutates it.
- **Fields are borrowed separately; indices are not.** `f(mut s.left, s.right)`
  is allowed. `f(mut a[i], a[j])` is an error whose fix suggests `swap(mut a,
  i, j)` or `a.pair_mut(i, j)`, which checks `i != j` once.
- **Two-phase borrows.** `a.push(a.len)` works, because the mutable borrow
  starts only at the call.
- **No struct holds a borrow.** A helper over an outside array is a top-level
  fn reading the file's `let`s, or a method that takes the array as a
  parameter. The research found this pattern in about 2% of corpus files;
  supporting it would bring lifetimes back for everyone.
- **Inline closures reuse the caller's borrow.** A lambda passed to a
  parameter marked `inline` is checked as part of the caller's body, so it
  may touch state that the call has borrowed mutably. That makes `undo` an
  ordinary library fn.
- **Generator-lent values** live for one loop iteration. Mutating the source
  inside the loop body is an error.

## `defer` (proposed)

`defer stmt` runs `stmt` on every exit from the enclosing block (normal end,
`return`, `break`, `?`), in reverse order of registration. It compiles to
copies of the statement on each exit edge, with nothing tracked at run time.

## Rollback (proposed)

Many algorithms need to undo changes: offline dynamic connectivity, DFS over
a tree of states, Mo's algorithm with rollback, divide and conquer over
queries. The design makes it explicit and cheap.

- **The `Rollback` trait:** `mark() -> Mark` and `undo_to(mark)`. Stdlib
  types implement it with their own journals, for example a rollback DSU
  (union by size, no path compression), and Fenwick and segment trees with a
  history.
- **`Journaled<[D]T>`:** an array that records old values on writes. It is a
  separate type, so the cost is visible.
- **`undo` blocks** list exactly what to restore:

  ```
  undo dsu, best:
      dsu.union(a, b)
      best max= dsu.size(a)
      recurse(child)
  // dsu and best are restored here
  ```

  Scalars are snapshotted, and anything else must implement `Rollback`.
  Blocks nest. Because state must be listed, journaling never happens
  silently.

## Open

- **Moves out of arrays (5.5):** the lean is `swap` and `take` (which leaves
  the default value) and no partial moves.
- **`@memo` control (5.9):** the table layout comes from the key type,
  `.clear()` is explicit, and evaluation order is the normal call order.
