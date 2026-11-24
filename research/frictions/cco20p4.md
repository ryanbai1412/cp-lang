# Friction report: cco20p4

## Dream

`dream.cp` (unverified, unchanged) reads the roads as `read lower_triangle [V, V]Colour` and
looks for an Euler tour per colour. The port keeps the proven per-start construction, which
is O(n^2) in all.

## Changes

The v0.2 port stored `red: [City, City]Bool` and the part colour as a `Bool` (`x`, "red when
true"). The rewrite reads into `[City, City]Colour` with an enum, so every comparison names
the colour, iterates `City if u != s` instead of `if u == s: continue`, binds the junction
with `if let a = first.last` instead of `first.len > 0` plus `first.last!`, and prints the
first part with `first.rev` instead of draining it.

## Remaining friction

- No reader for a lower-triangular matrix: each row is read as a `Str` and mirrored by hand.
- An enum cannot declare a method, so flipping a colour is spelled inline.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++ (by the checker).
Corpus tests pass.
