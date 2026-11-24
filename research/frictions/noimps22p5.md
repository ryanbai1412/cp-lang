# Friction Report: noimps22p5

The v0.2 port kept one `Set<(Int, Int)>` bag with a serial number per weight and a mode
integer for the cost rules. Main's later rewrite split it into a `MaxHeap` sweep and a `MinHeap`
sweep, but its heavy sweep kept the lightest weight ever seen, started from the sentinel
`w.max()!`, and assumed the bag never empties. It does empty whenever a level holds the last
bracket in the bag, as in `(()())` with weights `1 5 7`: main printed 16 where the accepted C++
prints 12. Now `least` is `?Int`, reset when the bag empties, the weights read as `[n]`, and the
Bool sweep options are named at each call. tests.toml gains that case and `--max`.

## The lightest of a max-heap is kept by hand
- **Where:** `least` beside `MaxHeap<Int>` in `sweep_heavy`.
- **Ideal:** a std double-ended heap (`MinMaxHeap`) with `.min` and `.pop_max`.

## Dream
```cp
fn sweep_heavy(escort: Bool) -> Int:
    var bag = MinMaxHeap<Int>()
    sum(for d in 0..n:
        bag.extend(level[d])
        let paid = if escort: bag.sum + (bag.len - 2) * bag.min! else: 0
        bag.pop_max()
        paid + (if escort: 0 else: bag.sum))
```

- Tokens: port 442, original 1055
- Max input (`gen.py --max`): port 0.18 s, 26592 KB, original 0.34 s, 46316 KB
