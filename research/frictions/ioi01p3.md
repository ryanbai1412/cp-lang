# Friction Report: ioi01p3

## Dream

```cp
@memo(reset: completions)                 // a memo cleared between questions
fn ways(stairs: [Row]0..=5) -> Int = ...  // the staircase itself as the key
```

## Gaps

- **The memo is kept by hand.** `ways` depends on the pinned letters, so it is
  cleared before each count; `@memo` has no reset, so the port keeps
  `memo: [Key]?Int` and clears it in `completions`. Severity: medium.
- **The staircase is packed base 6 into `Key`.** A memo keyed by five row
  lengths would drop `weight` and the `/ % 6` unpacking. Severity: low.
