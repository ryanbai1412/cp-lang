# Friction report: dmopc20c4p3

No gap: the port is the dream (a height sweep with std `compress` and `Fenwick`).

## Measurements

gen.py --max (n = m = 2e5): port 0.12 s, accepted C++ 0.73 s. Both lists sort in place, so nothing is copied.
