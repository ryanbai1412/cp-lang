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

## The judge's function task has no CP Lang form
- **Where:** the port reads stdin; `corpus/problems/ioi14p5/orig.cpp` calls the accepted
  `findSample(int n, int confidence[], int host[], int protocol[])` from a sample grader. The
  dream's `@grader fn findSample` over `[P]` and `[Joiner]` arrays is blocked (the same E0501
  as ioi14p1).
- **General fix:** grader fns over C arrays, with lengths from other params.
- **Owner:** core.
- **Severity:** high (the port can't be submitted to the judge as written).
