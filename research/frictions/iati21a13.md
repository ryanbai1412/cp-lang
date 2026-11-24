# iati21a13: News

## Dream

```cp
for v in V where knows[v] == 0 and let f = first[v]: wake[f].push(v)
var told = DynamicOrderTree([t.Pre] p => if knows[t.at[p]] == 1: Some(t.depth(t.at[p])) else: None)
```

## `if` can't combine a condition with `let`
- **Where:** the wake-up loop nests `if let f = first[v]:` and `if knows[v] == 0:`;
  `if knows[v] == 0 and let f = first[v]:` is E0103.
- **General fix:** let-chains in `if`/`while` conditions.
- **Severity:** medium (every optional plus a guard costs a nesting level).

## A vertex that doesn't know yet has depth `n`
- **Where:** `told`: unknown vertices hold depth `n`, beyond every count bound,
  because `DynamicOrderTree` counts plain `Int` values.
- **General fix:** `DynamicOrderTree<?T>` whose `count_lt` skips `None`.
- **Severity:** low.
