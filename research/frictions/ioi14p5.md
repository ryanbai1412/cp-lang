# ioi14p5: Friend

## Dream

```cp
enum Protocol = Host | Friends | Both  // read as 0, 1, 2
@grader fn findSample(n: 2..=1e5, confidence: [P]1..=1e4, host: [Joiner]P,
                      protocol: [Joiner]Protocol) -> Int:
    var best: [P](pick: Int, skip: Int) = [P] p => (pick: confidence[p], skip: 0)
    for j in Joiner.rev:
        ...
    best[P.first!].max()
```

## Integer-coded protocols can't read into an enum
- **Where:** `how: 0..=2` and the `if how == 0 / elif how == 1` chain.
- **General fix:** reading an enum by its ordinal (shared with the command
  frictions).
- **Severity:** low.

## Joiners are a subdomain of people
- **Where:** `domain Joiner = 1..n` and `P(j.label)`; a subdomain `Joiner <: P`
  would make `joins[j]` and `pick[j]` both typecheck without the conversion.
- **Severity:** low.
