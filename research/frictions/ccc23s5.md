# Friction Report: ccc23s5

## A recursive walk collects into a global list
- **Where:** `var kept` and `fn keep(l, r, k, b)` pushing into it.
- **What felt wrong:** the walk is a generator of kept numerators; without recursive `yields`
  (delegating to a nested generator) it mutates a list captured from the top level.
- **Goal it hurts:** brevity, locality.
- **Ideal code:** `fn keep(...) -> yields 0..=n` with `yield from keep(...)` for the two thirds,
  then `for x in keep(0, n, 0, 1): print(x)`.
- **General fix:** recursive generator delegation.
- **Severity:** low

## Cycle detection in a long division by hand
- **Where:** `fn in_cantor`: a tortoise and hare over the remainders.
- **What felt wrong:** the ternary expansion of `x / n` is an eventually periodic sequence;
  "walk every state until one repeats" is a std idea written by hand. The v0.2 port kept a
  `Set` of remainders (0.56 s at max); Floyd's walk needs no memory and runs in 0.15 s.
- **Goal it hurts:** brevity.
- **General fix:** `iterate(f, from:).until_repeat()` yielding each state once, implemented
  with Floyd or Brent so it allocates nothing.
- **Severity:** low

## Dream

```cp
let n: 2..=1e9 = read()
fn in_cantor(x: 0..=n) -> Bool =
    iterate(r => r % n * 3, from: x).until_repeat().all(r => r / n != 1 or r % n == 0)
fn keep(l: 0..=n, r: 0..=n, k: Int, b: Int) -> yields 0..=n:
    if r - l <= 10:
        for x in l..=r if in_cantor(x): yield x
        return
    yield from keep(l, n * (3 * k + 1) / (3 * b), 3 * k, 3 * b)
    yield from keep((n * (3 * k + 2) + 3 * b - 1) / (3 * b), r, 3 * k + 2, 3 * b)
for x in keep(0, n, 0, 1): print(x)
```

## Measurements

## Measurements

- Tokens: port 263, original 511
- Max input (`gen.py --max`): port 0.24 s, 7116 KB, original 0.34 s, 7524 KB
- Emitted C++: 11809 bytes
