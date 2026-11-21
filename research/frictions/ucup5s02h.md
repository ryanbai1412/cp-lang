# ucup5s02h

- **Source:** Universal Cup 5, stage 2 H (`proto/ports/ecnerwala/ucup5s02h_orig.cpp`)
- **Port:** `proto/ports/ecnerwala/ucup5s02h.cp`
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp`
- **Status:** Ported (`gate.sh ucup5s02h 300` passes; random generator added)

## No exact fraction value type

- **Goal it hurts:** express, hard to get wrong
- **Severity:** medium
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp:18, 23-24, 57, 66-70`
- **Actual:** `proto/ports/ecnerwala/ucup5s02h.cp:57-62, 111-121`
- **General fix:** an ordered `Frac` with cross-multiplied comparison, `min`/`max`, `inv` and reduced printing, so `d^2 / (q q')` is one value instead of a `(num, den)` pair with a hand comparator and a hand gcd print.

## Union-find without component ids or a count

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp:55-56, 64`
- **Actual:** `proto/ports/ecnerwala/ucup5s02h.cp:80-86`
- **General fix:** `components` (a count kept by `unite`), `component_ids()` (a dense `[V]Int` over roots) and a `unite` that reports whether it joined two sets, shared by `Dsu` and `ParityDsu`. Today the loop tests `all(same(..))` per round, builds `comp` by hand, and guards `unite` with `same`.

## No sum domain for an extra node

- **Goal it hurts:** hard to get wrong
- **Severity:** low
- **Area:** types
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp:45, 49-53`
- **Actual:** `proto/ports/ecnerwala/ucup5s02h.cp:73-78`
- **General fix:** a `domain Node = C | Pin` sum of a charge domain and a singleton, so the reference node is not position `n` of `0..=n` and charges are not converted with `Node.at(i)` at every call.

## Optional enum input without sentinels

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp:20-21, 46-48`
- **Actual:** `proto/ports/ecnerwala/ucup5s02h.cp:66-71, 76-77`
- **General fix:** reading a `?Side` from a letter token (any other letter is `none`), and reading a record table straight into a sorted one; the port compares `pin` with `'A'` and `'B'` by hand.

## No "all other groups" query

- **Goal it hurts:** express
- **Severity:** medium
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp:57-63`
- **Actual:** `proto/ports/ecnerwala/ucup5s02h.cp:89-108`
- **General fix:** a segment tree over groups whose node type is any user structure (here a `Hull`) with `others(of: g)` walking the siblings of `g`'s leaf path, plus a Boruvka driver that runs a sweep per round and returns each component's best outgoing edge. The port climbs `node ^ 1` and `node >>= 1` by hand twice and writes the mirrored sweep as a `sign` parameter.

## List edits by predicate

- **Goal it hurts:** express
- **Severity:** low
- **Area:** stdlib
- **Status:** open
- **Dream:** `proto/ports/ecnerwala/dream/ucup5s02h.cp:31-41`
- **Actual:** `proto/ports/ecnerwala/ucup5s02h.cp:39-50`
- **General fix:** `pop_while`, negative indexing (`pts[-2]`) and `get(i) -> ?T`, so the hull's pop loop and `idx == pts.len` check are one call each.
