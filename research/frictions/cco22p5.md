# cco22p5 friction

The old port hand-rolled both companies' union-find (`rep_a`/`mem_a`, `rep_b`/`mem_b`),
logged B's relabels in three parallel lists, encoded a house pair as `a * (n + 1) + b`,
and failed `cpl check` (E0603 on `let x = mem_b[u][idx]` then `ch_v.push(x)`). The port
keeps each company as a `Net` over `Dsu<House>` (B with `rollback: true`, undone with
`mark`/`undo_to`), keys the overlap `HashMap` by the `(House, House)` group pair, and
prints the optional answer with `none: -1`.

## Dream

```cp
struct Net:
    var dsu: Dsu<House>
    var members: [House]List<House>
    var pairs: Int
    fn link(mut self, u: House, v: House) -> List<House>:
        if let (kept, merged) = dsu.merge(u, v):
            let moved = members[merged].take()
            pairs += moved.len * members[kept].len
            members[kept] += moved
            return moved
        []
fn alone(rollback: Bool) -> Net = Net(Dsu<House>.new(rollback), [House] h => [h], 0)
```

## Moving a list out of an array cell copies it

- **Where:** `Net.link` moves the merged group's houses to the kept group and returns
  them. `let moved = members[merged]` lends the cell, so clearing it is E0602 and
  returning `moved` is E0603; there is no `List.take()` (the owned-move that
  `SortedBag`, `Deque`, `Heap` and `FlatMap` already have), so the port writes
  `members[merged].clone()` then `.clear()`: one extra copy per merge.
- **Repro:** `var xs: [V]List<Int> = [V] _ => [1]` / `let m = xs[V.first!]` /
  `xs[V.first!] = []` is E0602.
- **General fix:** `List.take(mut self) -> List<T>` like the other containers.
- **Status:** open.

## Mark is not in the prelude

- `history: List<(mark: Mark, ...)>` needs `use std.dsu.{Mark}`, though `Dsu.mark()`
  is reachable without an import. Naming a type a prelude method returns should not
  need a `use`.
- **Status:** open.

Measured: `cpl stress` agrees with the accepted C++ (4801885) on 30 cases (size 10) and 40
cases (size 40); on the `--max` case (N = A = B = 2e5) the outputs match, 0.34 s against
the C++'s 0.94 s.
