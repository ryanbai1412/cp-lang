# Friction report: egoi22p2

No gap in the language. The v0.2 port ran only the O(w^2) split recurrence, about 2.5e11 steps at w = 5e5 (h = 1). The v0.3 port also has the O(w h^2) cut-profile DP for w > 1e4, where h <= 50, as the accepted C++ does, with `Comb<Z>` for the binomials. Stress against the accepted C++ covers w in 10001..20000.

## Measurements

| case | port | accepted C++ |
|---|---|---|
| w = 1e4, h = 50 (gen.py --max) | 0.19 s | agrees |
| w = 10001, h = 49 | 0.06 s | agrees |
| w = 5e5, h = 1 | 0.02 s | agrees |
| w = 2.5e5, h = 2 | 0.02 s | agrees |
