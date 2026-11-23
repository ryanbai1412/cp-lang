# Friction Report: noi17p2

## 1. Length-carrying `Hash` value with `Hash.empty` and `+` concatenation

- **Area:** strings
- **Goal:** hard-to-get-wrong
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_10/dream/noi17p2.cp:16,32,35,46-47`
- **Port:** `proto/ports/scout_batch_10/noi17p2.cp:18-24,43,53-54`

`std.strings.HashVal` does not carry its length and `RollingHash.combine(a, b, len_b)` requires passing `len_b` by hand (plus constructing a dummy `RollingHash.new(List.filled(longest, 0))` just to borrow bases and writing `HashVal(0, 0)` for the empty hash). The port wraps `struct Block(hash: HashVal, len: Int)` with `concat(a, b)` so lengths cannot be miscounted.

## Nested map over a window traps

- **Where:** spec.cp's ask, first written `s.windows(k).map(w => Z(seen.get(hasher.hash_of(w.map(c => c.digit()))))).prod()`.
- **What felt wrong:** both engines trap R0001 "rekey needs as many keys as cells: 0..5 for 0 cells" on `w.map(...)` inside the lazy `map`; the same code in a `for` loop runs.
- **Goal it hurts:** express (the obvious spelling fails).
- **Ideal code:** the line above.
- **General fix:** a window yielded into a lazy map keeps its cells when mapped again (compiler lane; repro in its worklog).
- **Severity:** medium. **Status:** open (routed to compiler). spec.cp now hashes `s` once and reads `h.get(at..at + k)`, which is also O(|s|) per ask instead of O(k|s|).

## `RollingHash.combine` needs a pre-sized hash for its powers

- **Where:** `let hasher = RollingHash.new(List.filled(longest, 0))` in spec.cp.
- **What felt wrong:** `combine` documents growing its power table on demand but traps R0101 past `len`, so the hasher carrying the shared bases must be padded to the longest window.
- **Goal it hurts:** hard-to-get-wrong.
- **General fix:** `combine` grows the powers as documented (stdlib lane; repro in its worklog), or the length-carrying hash of section 1.
- **Severity:** low. **Status:** open (routed to stdlib).
