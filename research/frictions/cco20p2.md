# Friction Report: cco20p2

Already at bar apart from the bound on `n` (now `1..=2e5`): days and exercises are separate
domains, the buckets are `[Day]List<Ex>`, the heap and Fenwick are std, and failure is `?Int`.

## Bucketing by a key is spelled by hand
- **Where:** `var due_on: [Day]List<Ex> = []` then a push loop.
- **Ideal:** `let due_on = Ex.buckets(over: Day, by: e => due[e])`.

## Dream
```cp
let n: 1..=2e5 = read()
domain Ex = 1..=n
domain Day = 1..=n
let due: [Ex]Day = read()
let due_on = Ex.buckets(over: Day, by: e => due[e])
fn moves() -> ?Int:
    var free = MaxHeap<Ex>()
    var placed = Fenwick<Ex, Int>()
    var total = 0
    for day in Day.rev:
        free.extend(due_on[day])
        let e = free.pop()?
        total += placed.prefix(e.l)
        placed.add(e, 1)
    total
print(moves() ?? -1)
```

- Tokens: port 167, original 507
- Max input (`gen.py --max`): port 0.03 s, 14492 KB, original 0.12 s, 26624 KB
