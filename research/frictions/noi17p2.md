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

## v0.3 re-port

Port moved toward the dream: the block counts are a `Counter<HashVal>` (no `get ?? 0`
read-modify-write), the links are `before`/`after` instead of the swapped `ahead`/`behind`, the
op kind and k are typed reads, and a query is `map(...).prod()`.

The dream keeps an `Op` enum read as `read<[q]Op>()` and a `PolyHash` value built with
`PolyHash.of(lengths)` and `h.push(len)`; std has `RollingHash` over a fixed list with
`combine`, so the port carries a `Block(hash, len)` pair and its own `concat`.

### Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 2e5, m = 5e5, sum |s| ~ 1e7)
2.77 s vs the v0.2 port 2.56 s vs C++ 0.96 s, same output. The gap to C++ is the hash map of
HashVal pairs (C++ uses one 64-bit key in a gp_hash_table).

## Max case

Blocks are keyed by a 61-bit `Hash61` sharing `rh.base`, not a two-word `HashVal`. At --max
that is 1.87 s and 857 MB peak, down from 3.84 s and 1.25 GB; the accepted C++ takes 0.84 s and
280 MB. What remains is the std hash table's layout (load at most 1/2, 17 bytes per slot,
both arrays live while it rehashes), sent to Content stdlib with this input as the repro.
