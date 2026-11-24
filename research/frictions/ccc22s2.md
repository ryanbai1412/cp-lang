# Friction Report: ccc22s2

## A group of three names is three assignments
- **Where:** `group_of[groups[k].a] = k` / `.b` / `.c`.
- **What felt wrong:** "every name of group k maps to k" reads the record's fields one by one;
  destructuring them into a list (`for s in [a, b, c]`) copies the three strings.
- **Goal it hurts:** brevity; costs are visible.
- **Ideal code:** `let group_of = Map(groups.enumerate().flat_map((k, names) => names.map(s => (s, k))))`
  with `groups: [Group][3]Str` lent, not copied.
- **General fix:** read fixed-size rows as `[3]Str` arrays, and lend record fields through
  `for s in row` without copies.
- **Severity:** low

## Dream

```cp
let together: [read()](a: Str, b: Str) = read()
let apart: [read()](a: Str, b: Str) = read()
let groups: [domain Group = read()][3]Str = read()
let group_of = Map((s, k) for k in Group for s in groups[k])
fn same(p: (a: Str, b: Str)) -> Bool = group_of[p.a] == group_of[p.b]
print(together.count(p => not same(p)) + apart.count(same))
```

## Measurements

- Tokens: port 209, original 451
- Max input (`gen.py --max`): port 0.12 s, 89352 KB, original 0.08 s, 11392 KB
- Emitted C++: 15772 bytes
