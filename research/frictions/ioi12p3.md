# ioi12p3: Crayfish Scrivener

## Dream

```cp
enum Op = T(letter: Char) | U(x: 1..=n) | P(i: 0..n)
let texts = PersistentStack<Char>()  // each command names a version
...
```

## Commands decode by hand
- **Where:** `op: Char` and the `if op == 'T' / elif op == 'U'` chain read
  each operand by hand; a tagged-enum `read` would carry the bounds per arm.
- **Severity:** low.

## The skew-binary ancestor jump is hand-rolled
- **Where:** `up`/`jump` and the climb loop: an O(1)-insert, O(log n)
  level-ancestor structure. A std `JumpTree` (or `Rooted.ancestor_at_depth`
  for a growing tree) would replace both arrays and the loop.
- **Severity:** medium.

## The judge's function task has no CP Lang form
- **Where:** the port reads stdin; `orig.cpp` wraps submission 4682651 in a
  sample grader (`Init`, `TypeLetter`, `UndoCommands`, `GetLetter`).
- **Severity:** high.
