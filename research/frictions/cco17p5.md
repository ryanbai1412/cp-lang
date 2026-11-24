# Friction Report: cco17p5

## Bucketing by a key is spelled by hand
- **Where:** `var by_need: [0..=n]List<0..=1e4> = []` then a push loop.
- **What felt wrong:** "the prices grouped by need" is one idea written as a declaration and a loop.
- **Ideal:** `let by_need = people.buckets(over: 0..=n, by: p => p.need, of: p => p.price)`.

## Dream
```cp
let n: 1..=2e5 = read()
let people: [n](need: 0..=n, price: 0..=1e4) = read()
let by_need = people.buckets(over: 0..=n, by: p => p.need, of: p => p.price)
var waiting = MinHeap<Int>()
var paid = 0
for k in (0..=n).rev:
    waiting.extend(by_need[k])
    while n - waiting.len < k: paid += waiting.pop()!
print(paid)
```

- Tokens: port 140, original 625
- Max input (`gen.py --max`): port 0.03 s, 17284 KB, original 0.03 s, 15884 KB
