# Friction report: dmopc20c4p3

No gap: a height sweep with std `compress` and `Fenwick`, the cords popped by std `pop_while` as each Roomba is reached.

## Dream

The port as written. The cords sit in a list sorted tallest last, so each Roomba pops the cords at least its height with std `pop_while` and adds them to the `Fenwick`; no cursor index.

## Measurements

gen.py --max (n = m = 2e5): port 0.12 s, accepted C++ 0.73 s. Both lists sort in place, so nothing is copied.
