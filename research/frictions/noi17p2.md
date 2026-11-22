# Friction Report: noi17p2

## 1. Length-carrying `Hash` value with `Hash.empty` and `+` concatenation

- **Area:** strings
- **Goal:** hard-to-get-wrong
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_10/dream/noi17p2.cp:16,32,35,46-47`
- **Port:** `proto/ports/scout_batch_10/noi17p2.cp:18-24,43,53-54`

`std.strings.HashVal` does not carry its length and `RollingHash.combine(a, b, len_b)` requires passing `len_b` by hand (plus constructing a dummy `RollingHash.new(List.filled(longest, 0))` just to borrow bases and writing `HashVal(0, 0)` for the empty hash). The port wraps `struct Block(hash: HashVal, len: Int)` with `concat(a, b)` so lengths cannot be miscounted.
