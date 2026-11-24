# Friction report: coci25c3p2

Rethought. The v0.2 port rolled the Stirling recurrence by hand over a `[0..=n]Z` row. The v0.3 port reads the answer off std's `stirling1_row<P>(n)`, which is the same count.

No language gap.

## Measurements

gen.py --max (n = 5000): port 0.04 s, accepted C++ 0.49 s, same output.
