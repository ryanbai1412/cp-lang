# Friction report: noip19p4

There is no dream.cp; the ideal is below. The port is the editorial's complement count:
- all sets are `prod(row + 1) - 1`;
- for each ingredient j, a DP over methods on `Lead = -n..=n` (dishes with j minus the others) counts the bad sets;
- the DP only touches leads with |lead| < i before method i.

## Dream

```cp
for j in Food:
    var ways = [Lead] d => Z(if d.label == 0: 1 else: 0)
    for i in Method:
        ways = [Lead] d => ways[d] + (ways[d.prev?] ?? 0) * a[i, j] + (ways[d.next?] ?? 0) * (row[i] - a[i, j])
    bad += sum(ways[d] for d in Lead if d.label > 0)
```

## The rolling DP is a clone plus a hand-limited range
- **Where:** lines 29-34: `var next = ways.clone()`, then `for d in Lead(1 - i.label)..=Lead(i.label - 1)` pushing into `d.next`/`d.prev`.
- **What felt wrong:** "the new row from the old" needs an explicit copy (`clone`, a full O(n) copy per method that the dream's comprehension also pays) and a range rebuilt from labels so that the cost is O(i), not O(n).
- **Goal it hurts:** express.
- **Ideal code:** the dream's comprehension, restricted to `Lead.within(i.label)` or a `Lead.around(0, radius: i.label)` range.
- **General fix:** none needed for correctness or speed (0.21 s against the C++ 0.25 s). A std range constructor centred on a domain value would remove the label arithmetic. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 249, accepted C++ 832 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.21 s / 3 MB, accepted C++ 0.25 s / 4 MB, with the same output.
- **Emitted C++:** 18999 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
