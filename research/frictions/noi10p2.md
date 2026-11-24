# Friction Report: noi10p2

## 1. `split_best` is built in full before an early `break`
- **Where:** the dream's `split_best(Note.cuts, range: s => s.shift(len), best).take(k).sum()`.
- **What felt wrong:** the generator is compiled to a `std::vector` of every yielded value, so
  `take(k)` or a `break` after k values still enumerates every chord (about n (R - L)): 60 s+ on
  the max case. The port keeps an explicit `MaxHeap<Chord>` doing the same splits by hand.
- **Goal it hurts:** the obvious way is the fast way.
- **Ideal code:** see Dream.
- **General fix:** lazy `yields` generators under `for ... break` and `.take(k)` (sent to Core).
- **Severity:** high (TLE).

## Dream
```cp
let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], c))
fn best(from: Note.cuts, ends: Range<Note.cuts>) -> (Int, Note.cuts):
    let (p, at) = peak.fold(ends)
    (p - pre[from], at)
print(split_best(Note.cuts, range: s => s.shift(len), best).take(k).sum())
```

## Measurements
Tests 11/11. Stress: 30/30 agree with the v0.2 port and with AC; max (n = k = 5e5) 0.17 s vs
C++ 0.25 s, same output.
