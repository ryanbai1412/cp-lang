# Friction Report: ioi05p6

## Dream

```cp
let river = Tree.parents(root: Bytetown, up: rows.down)   // a rooted tree from parent links
@memo fn cost(v: Village, mill: Site) -> Row = ...
print(river.children(root).min_plus(c => cost(c, root))[k])
```

## Gaps

- **Bytetown and the villages are two domains glued by hand.** `Site = 0..=n`
  with `site`/`village` converters exists because a tree rooted outside the
  village domain has no std shape. Severity: medium.
- **Min-plus merging of child rows is written out.** `spread` is the knapsack
  `(min, +)` convolution over `Mills`; std has no min-plus convolution of
  `[Mills]Int` rows. Severity: low.
- **"Every village drains to Bytetown" is not in the read type.** A parent
  read can't state acyclicity. Severity: low.
