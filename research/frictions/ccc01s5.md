# Friction Report: ccc01s5

## Search with an undo stack
- **Where:** `picked.push(p)` ... `picked.pop()` around the recursive `search`.
- **What felt wrong:** the answer list is a global stack pushed and popped by hand around each
  try; returning the list from the search would need no undo.
- **Goal it hurts:** common mistakes hard to write.
- **Ideal code:** `fn search(ahead, top) -> ?List<Pair>` with
  `Pair.find_map(p => ... search(rest).map(l => [p] + l))`.
- **General fix:** `find_map` on domains and cheap list prepend (a persistent list) for
  search results.
- **Severity:** low

## Dream

```cp
let (m, n): (1..=40, 1..=40) = read()
domain Pair = 1..=n
let a: [Pair]Str = read()
let b: [Pair]Str = read()
fn search(ahead: Str, top: Bool, left: Int) -> ?List<Pair>:
    if left == 0: return None
    Pair.find_map(p =>
        let (long, short) = if top: (ahead + a[p], b[p]) else: (ahead + b[p], a[p])
        if long.starts_with(short):
            let rest = long[short.len..]
            if rest.is_empty: Some([p]) else: search(rest, top, left - 1).map(l => [p] + l)
        elif short.starts_with(long): search(short[long.len..], not top, left - 1).map(l => [p] + l)
        else: None)
if let l = search("", true, m - 1): print(l.len); for p in l: print(p) else: print("No solution.")
```

## Measurements

- Tokens: port 256, original 494
- Max input (`gen.py --max`): port 0.00 s, 3200 KB, original 0.03 s, 3456 KB
- Emitted C++: 14250 bytes
