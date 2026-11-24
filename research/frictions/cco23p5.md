# cco23p5 friction

The old port kept raw `Int` cities with `-1` sentinels (`memo1`, `memo2`, `b1 = -1`, `neg`),
spelled the K = 2 dp's four (done, back) states as `0..4` codes, wrote every route shape
twice (once in `dp2`, once in `build2`) and failed `cpl check` (E0603 on
`let v = adj[u][idx]` then `ch[u].push(v)`). The port roots the tree with `Rooted`, memoises
`best(u, done, back)` with `@memo(array)`, and writes each route shape once, as a list of
`Part`s that both the value and the replay read. The route is not unique, so `checker.py`
judges it.

## Dream

```cp
@memo(array)
fn best(u: City, done: Bool, back: Bool) -> (gain: Int, parts: List<Part>) = ...
// the route replays best(...).parts; no second pass over the shapes
```

## Keeping the plan beside the value

- **Where:** `best` returns only the value, and `replay` calls `plans` again to find the
  argmax. A memoised function returning `(Int, List<Part>)` would keep every plan alive;
  splitting value and plan costs one extra `plans` call per city on the route.
- **General fix:** none needed in the language; noted for the reviewer.
- **Status:** open (design note).

## `cpl check` took 33 s on one closure shape

- **Where:** an earlier draft of `named` wrote
  `kinds.map(kd => t.children(u).collect().sorted(by: v => -extra(v, kd.done, kd.back)).take(3).collect())`
  inside the mutually recursive `best`/`plans`/`named` group; `cpl check` took 33 s on the
  file. Moving the sort into its own `top3` function brought it to 0.2 s. A 5-line copy of
  the closure outside the recursive group checks in 0.5 s, so the cost needs the recursion.
- **Status:** open; reported to Core with the shape above.

## Speed

- The port takes 0.48 s on the `--max` case (N = 2e5, K = 2) against the accepted C++'s
  0.28 s (1.7x, over R10's 1.1x). The C++ enumerates the top-3 combinations in fixed
  arrays; `named` builds them as small lists, and every plan is a small `List<Part>`.
  An earlier draft that kept one plan per child took 1.6 s.
- **General fix:** small fixed-capacity lists (inline storage) for `p.kids + [v]`-style
  combination building.
- **Status:** open.

Measured: `cpl stress` with `checker.py` agrees with the accepted C++ (5580402) on 30 cases
(size 10) and 60 cases (size 12); `checker.py` accepts the port's `--max` output.
