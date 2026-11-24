# yac1p4

Score: 3/5
Dream: 3/5

Follow-up: each DP step is now `swap(below, reach[open])` and then two
`reach[open].union_shifted(...)` (b3101637a), so no step allocates or builds a temporary. At
--max the native build takes 0.15 s, against 0.24 s before; all-Z n = 250 takes 0.31 s, against
0.43 s. The Bitset recount friction is fixed in std, so its friction file is removed.

## Cold review (HEAD)

Port 3/5 (no dream.cp), dream 3/5. One `Bitset<Lifted>` per count of open right ends, updated in place with `swap` and `union_shifted` (lines 22-31). It runs at 0.36x the accepted C++'s time.
- The lifted encoding (lines 21-31, and `t.ord - whole` on line 35) exists only because `union_shifted` shifts up. A signed shift, or a Bitset over a signed domain, would store totals as they are (Owner: stdlib).
- `(c - 'A' + 1) * (c - 'A' + 1)` (line 17) is character arithmetic, squared by repetition (Owner: idioms/stdlib, in the friction).
- `Open.first!..=Open(min(i, n - i))` (line 28) builds a domain prefix from an `Int` (Owner: stdlib, in the friction).
