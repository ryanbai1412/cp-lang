# Friction Report: cco18p6

The v0.2 port read both permutations through push loops shifted to 0-based, kept the flops in
two parallel lists `ox`/`oy`, copied a's flops element by element before reusing the globals for
b, wrapped `first_true` in an `lb` helper, and printed b's flops with a hand-written reverse
`while`. Now `sorting_flops()` reads one permutation typed `[n]Val` and returns its flops as a
`List<Flop>`, the merge finds its cut points with `first_true` directly and takes the median of
`cur[l..r].sorted()`, and the output walks `there` then `back.rev`. The package gains a checker
that replays the flops, since many flop sequences are correct.

## The sort state is global
- **Where:** `var cur` and `var flops` at top level, shared by `reverse`, `merge` and
  `sort_range`.
- **Ideal:** the three as local helpers over a `mut` slice inside `sorting_flops`, without
  becoming `std::function`.

## A lent parameter cannot become the global state
- **Where:** `sorting_flops` reads its own permutation, because `cur = p` from a parameter is
  E0603.
- **Ideal:** an owned parameter (`fn sorting_flops(own p: List<Val>)`) so the caller keeps the reads.

## Dream
```cp
fn sorting_flops(own p: [n]Val) -> List<Flop>:
    var flops: List<Flop> = []
    p.merge_sort(merge: (run, mid) => run.rotate_by_reversals(mid, record: flops))
    flops
let (a, b): ([n]Val, [n]Val) = read()
let flops = sorting_flops(a) + sorting_flops(b).rev
print(flops.len)
for f in flops: print(f.l + 1, f.r + 1)
```

- Tokens: port 409, original 831
- Max input (`gen.py --max`): port 0.02 s, 8756 KB, original 0.02 s, 7368 KB
