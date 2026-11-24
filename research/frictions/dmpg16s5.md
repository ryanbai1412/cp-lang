# Friction report: dmpg16s5

Already at bar except one bound: tastes are `-1e8..=1e8` (the statement), not `-1e9..=1e9`. gen.py now stays inside the statement (it drew tastes up to 1e9) and always has a positive slice, as the statement promises.

## Measurements

gen.py --max (n = 1e5, k = n / 2): port 0.01 s, accepted C++ 0.02 s.
