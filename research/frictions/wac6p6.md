# Friction report: wac6p6

There is no dream.cp; the ideal is below. The port follows the three-case strategy in its header. A `[Colour]?Bowl` gives each colour's bowl, and `put` prints `PLACE b` or `DISCARD`.

## Dream

```cp
enum Colour: R, G, B          // read from its letter
...
let (a, b, c) = counts.sorted().tuple()
```

## Three letters become a domain through ordinals
- **Where:** lines 15-21: `domain Colour = 0..3  // R, G, B`, `let (red, green) = (Colour(0), Colour(1))`, and `cane()` mapping `'R'`, `'G'`, anything else to `Colour(2)`.
- **What felt wrong:** the colours are named by a comment and by position. The "else" arm reads any character as blue.
- **Goal it hurts:** hard-to-get-wrong, express.
- **Ideal code:** a three-value enum, usable as an array index and read from its letter, as the dream shows.
- **General fix:** enums whose variants index arrays and are read by name or letter. Owner: core/spec.
- **Severity:** low.

## A sorted array is unpacked by index
- **Where:** lines 27-28: `let sorted = counts.sorted()` and `let (a, b, c) = (sorted[0], sorted[1], sorted[2])`.
- **What felt wrong:** a fixed three-element sort is destructured by hand.
- **Goal it hurts:** express.
- **Ideal code:** `let (a, b, c) = counts.sorted().tuple()`, or a tuple pattern over a fixed-length list.
- **General fix:** destructuring of a list or array whose length the type knows. Owner: core.
- **Severity:** low.

## The next free bowl is counted
- **Where:** `Bowl(1 + Colour.count(y => bowl[y] != None))` in the second case.
- **What felt wrong:** "the next unused bowl" is computed from the ordinal of a count.
- **Goal it hurts:** express.
- **Ideal code:** `Bowl.find(b => not bowl.contains(Some(b)))!`, a first-match search over a domain, as `Bitset.find(pred)` already does over a set.
- **General fix:** a `find(pred) -> ?D` on every domain, matching `Bitset.find`. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 402, accepted C++ 617 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.01 s / 3 MB, with the same output.
- **Emitted C++:** 17209 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.

## Addendum: the reference and the statement sample
The accepted C++ reads exactly 100 canes per factory (`for (int i = 0; i < 100; i++)`). That is
the statement's guarantee (A + B + C = 100), so it is a valid reference. The statement's 10-cane
sample (case 01, `5 2 3`) breaks the guarantee, and under the interactor the reference gets
"unexpected EOF". Port and reference both pass the other 7 cases, and case 08 (`--max`) takes
90 ms for the port and 140 ms for the reference under `cpl test`. The port loops `a + b + c`
times, so it handles the sample. Case 01 is interactive and has no committed answer.
