# ioi04p4: Phidias

## Dream

```cp
@memo
fn waste(a: 1..=w, b: 1..=h) -> Int:
    if (a, b) in wanted: return 0
    min(a * b,
        (1..=a / 2).min(x => waste(x, b) + waste(a - x, b)),
        (1..=b / 2).min(y => waste(a, y) + waste(a, b - y)))
```

## An empty-range minimum needs a parenthesised fallback
- **Where:** `waste`: `.min() ?? (a * b)` twice. `??` next to `*` is E0110
  without the parentheses, and there is no variadic `min` that skips empty
  optionals, so each cut direction is bound to its own name.
- **General fix:** let `min(x, ?y, ...)` ignore `None` operands, or give
  `min(by:)` a `default:` argument.
- **Severity:** low.
