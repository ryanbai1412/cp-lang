// cpl 0.3.0
// Problem: count occurrences of pattern p in text t (CSES 1753).
// Input: t on line 1, p on line 2 (lowercase, lengths <= 1e6).
// Output: the number of positions where p occurs in t.
// Guesses: H-Z-KEYS (stub `z_function<S: Indexed, K>` leaves K unrelated to
// S; I assume it is `s.keys`, i.e. Int here), H-PRELUDE-MEMBERSHIP (not in
// std/prelude.cp, so imported).

use std.strings.{z_function}

let t = read<Str>()
let p = read<Str>()
let s = p + "#" + t
let z = z_function(s)
print(s[p.len + 1..].keys.count(i => z[i] >= p.len))
