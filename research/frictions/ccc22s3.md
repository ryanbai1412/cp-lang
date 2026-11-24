# Friction Report: ccc22s3

## Greedy fill with a filler value
- **Where:** `var tune: [Note]Pitch = Pitch.first!`, then `if take < 1: break`.
- **What felt wrong:** the array starts with a placeholder pitch that is only printed when the
  answer is -1 (and then it is not printed); building the tune as a fold that can fail would
  need no filler.
- **Goal it hurts:** no sentinels.
- **Ideal code:** `let tune = Note.try_build(i => ...)` returning `?[Note]Pitch`.
- **General fix:** a fallible array builder whose step reads the earlier entries.
- **Severity:** low

## Dream

```cp
let (n, m, k): (1..=1e6, 1..=1e6, 1..=1e18) = read()
domain Note = 0..n
domain Pitch = 1..=m
var left = k
let tune = Note.try_build((i, tune) =>
    let most = min(i.idx + 1, m)
    let take = min(most, left - (n - 1 - i.idx))
    guard take >= 1 else: None
    left -= take
    Some(if take == most: Pitch.nth(i.idx % m) else: tune[Note.at(i - take)!]))
print(if left == 0: tune else: None)  // None prints -1
```

## Measurements

- Tokens: port 166, original 388
- Max input (`gen.py --max`): port 0.01 s, 6656 KB, original 0.05 s, 7424 KB
- Emitted C++: 9720 bytes
