# Friction Report: cco05p6

The v0.2 port scanned the whole stack with a `?Int` position, then shifted the tail down by hand
(it carried a `WISH C3: stack.remove_at(i)`). Now `stack.keys.rfind(...)` finds the topmost hit,
`stack.remove(i)` takes it out and `push` puts it on top. The clicks are read as one typed list,
the counts are bounded types, and the package gains gen.py and tests.toml.

## `List.remove(i)` takes a position, which reads like a value
- **Where:** `let w = stack.remove(i)`; `remove(at: i)` is rejected, while sets and maps spell
  `remove(x)` for a value.
- **Ideal:** `stack.remove(at: i)` (or `take_at`) so the call shows it is positional.

## Dream
```cp
let _screen: (Int, Int) = read()
let n: 1..=5e4 = read()
domain Win = 1..=n
let frame: [Win](left: Int, top: Int, right: Int, bottom: Int) = read()
let m: 0..=5e4 = read()
var stack = Win.collect()
for (x, y) in (read(): [m](Int, Int)):
    let f = w => frame[w].left <= x <= frame[w].right and frame[w].bottom <= y <= frame[w].top
    if let i = stack.keys.rfind(i => f(stack[i])):
        print(stack.move_to_back(at: i))
    else: print(0)
```

- Tokens: port 211, original 448
- Max input (`gen.py --max`): port 0.00 s, 5800 KB, original 0.01 s, 5376 KB
