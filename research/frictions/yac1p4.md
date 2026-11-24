# Friction report: yac1p4

There is no dream.cp; the ideal is below. The port scans right to left with one `Bitset<Lifted>` of reachable totals per count of open right ends. Each total is lifted by the worth seen so far, so the sets only shift up, and `swap` plus `union_shifted` updates them in place.

## Dream

```cp
let worth = word.map(c => (c.ord_in('A'..='Z') + 1) ** 2)
for open in Open.through(min(i, n - i)): ...
```

## A letter's worth is character arithmetic
- **Where:** line 17: `(c - 'A' + 1) * (c - 'A' + 1)`.
- **What felt wrong:** "the letter's position in the alphabet" is spelled as a subtraction, and is computed twice in order to square it.
- **Goal it hurts:** express.
- **Ideal code:** `(c - 'A' + 1) ** 2` today. A named position (`c.ord_in('A'..='Z')`) would be better.
- **General fix:** the `** 2` is port spelling. Owner: idioms. A letter-position helper on `Char`: Owner: stdlib.
- **Severity:** low.

## A domain prefix is built from an `Int` bound
- **Where:** line 27: `Open.first!..=Open(min(i, n - i))`.
- **What felt wrong:** the loop bound is an integer, so it is converted into the domain by hand. `Open(x)` would panic if `x` left the domain, which it can't here.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `Open.through(min(i, n - i))`, a prefix of a domain up to an ordinal.
- **General fix:** a domain prefix by ordinal, shared with every "first k values" loop. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 223, accepted C++ 465 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.31 s / 12 MB, accepted C++ 0.85 s / 14 MB, with the same output.
- **Emitted C++:** 18234 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
