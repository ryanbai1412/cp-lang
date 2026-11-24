# cco21p3 friction

The old port kept raw `Int` passes, index lists and `List.filled` counters, and failed
`cpl check` (E0603 on `let v = adj[u][idx]` then `tour_node.push(v)`). The port now reads
each room's corridors as a `List<Room>`, types passes as `Pass`, takes the pass sizes and
their sums from `scan(+)` and finds the k-th joined step with `Fenwick.kth`.

## Dream

```cp
let back = doors.index_of(p)
let order = doors.after(back).map(c => (c, pass.next!)) + doors.before(back).map(c => (c, pass))
```

## No index-of on a list

- **Where:** `walk` needs the position of the parent's corridor in the room's list.
  `Seq.index_of` exists only on a compressed domain, and `first_true` on a range is a
  binary search (it traps on a non-monotone predicate, which is right but was the first
  thing tried), so the port writes `(0..around[u].len).find(i => around[u][i] == p)!` and walks
  the corridors by index from `back + 1`, wrapping, instead of building `order`.
- **Why by index:** binding `let doors = around[u]` in the recursive `walk` copied the list in
  every frame (`cpl explain`: "a call while the name lives may change what it reads", though
  `around` is a `let`), and the per-frame `order` lists were alive down the whole recursion: the
  `--max` case peaked at 306 MB. By index it peaks at 226 MB (accepted C++ 235 MB), 0.54 s.
- **General fix:** a linear `position(x) -> ?Int` (or `find_index`) on every `Seq`.
- **Status:** open.

## The checked-in port failed `cpl check`

- E0603 on a plain `V` read out of `[V]List<V>` and pushed: sent to Core with the repro
  `var xs: List<List<Int>> = [[1]]` / `let x = xs[0][0]` / `ys.push(x)`. The rewrite
  iterates the lists directly and does not depend on the fix.

Measured: `cpl stress` agrees with the accepted C++ (4552867) on 30 cases (size 10) and 40
cases (size 40); on the `--max` case (N = Q = 8e5) the outputs match, 0.63 s against the
C++'s 1.3 s.
