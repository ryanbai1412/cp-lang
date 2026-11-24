# Friction report: dmopc20c5p2

No gap: the port is the dream. It enumerates each row's crossed column range directly (std `ceil_div`), so it needs no set and no sort; the accepted C++ collects a set.

## Measurements

gen.py 1 1000000 (N = 999999, M = 1e6): port 0.10 s, accepted C++ 0.86 s.
