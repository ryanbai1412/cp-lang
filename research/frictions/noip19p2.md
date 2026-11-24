# Friction report: noip19p2

There is no dream.cp; the ideal is below. The port is the editorial's one-pass preorder:
- `open[v]`, the nearest unmatched `(`, and `ending[v]` come from the parent and from the matched bracket's parent;
- both are `?V`-typed, so the root needs no sentinel.

## Dream

```cp
let n: 1..=5e5 = read()
domain V = 1..=n
domain Child = 2..=n
let s: [V]Char = read()
let t = Rooted.from_parents(read<[Child]V>())
var open: [V]?V = None
var ending: [V]Int = 0
var k: [V]Int = 0
for v in t.at:
    open[v] = if s[v] == '(': Some(v) else: t.parent(v).and_then(a => open[a])
    if s[v] == ')': if let j = open[v]: ...
print(V.fold(v => v.label * k[v], ^))
```

## A parent table over a sub-domain needs a lambda with label arithmetic
- **Where:** line 18: `Rooted.from_parents([V] v => if v.label >= 2: boss[Child(v.label)] else: None)`.
- **What felt wrong:** the input is "the parent of each vertex 2..=n", already a `[Child]V`. Lifting it to `[V]?V` re-derives membership in `Child` from labels.
- **Goal it hurts:** express.
- **Ideal code:** `Rooted.from_parents(boss)` taking a `[Child]V` where `Child` is a sub-domain of `V`, or `boss.extend(to: V)` giving `[V]?V`.
- **General fix:** a std conversion from an array over a sub-domain to an optional array over the parent domain (`Child.at(v)` already maps the index). The parent-of-each-non-root input shape is common (btoi17p6, ioi09p7, ...). Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 242, accepted C++ 1171 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.07 s / 62 MB, accepted C++ 1.37 s / 139 MB, with the same output.
- **Emitted C++:** 23411 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
