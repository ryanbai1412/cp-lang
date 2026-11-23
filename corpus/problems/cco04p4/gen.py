"""cco04p4 generator: `seed size`; two closed orbits, each `x y m` then m <= 100 moves `d axis`.

Small sizes give short random orbits (some with d = 0 moves); 13 <= size < 10000 gives laps of
up to `size` unit steps in moves of up to 100; size >= 10000 gives two laps of about 1e4 unit
steps whose lengths share only the factor 2, so the joint period is about 5e7. Every lap has
positive length: the accepted reference indexes an empty lap out of bounds.
"""
import random
import sys

seed, size = int(sys.argv[1]), int(sys.argv[2]) if len(sys.argv) > 2 else 8
rng = random.Random(seed)


def parts(total, cap):
    out = []
    while total > 0:
        out.append(min(cap, total))
        total -= out[-1]
    return out


def orbit(half, cap, zeros=0):
    a = rng.randint(0, half)
    b = half - a
    moves = [(d, "X") for d in parts(a, cap)] + [(-d, "X") for d in parts(a, cap)]
    moves += [(d, "Y") for d in parts(b, cap)] + [(-d, "Y") for d in parts(b, cap)]
    moves += [(0, rng.choice("XY")) for _ in range(zeros)]
    rng.shuffle(moves)
    return moves


def emit(x, y, moves):
    assert len(moves) <= 100
    print(x, y, len(moves))
    for d, c in moves:
        print(d, c)


if size >= 10000:
    lens = [(10000, 9998), (9998, 10000), (9998, 9996), (10000, 9994), (9996, 9998)]
    l1, l2 = lens[seed % len(lens)]
    for lap in (l1, l2):
        half = lap // 2
        while True:  # 2 * (ceil(a / 100) + ceil(b / 100)) <= 100
            ms = orbit(half, 100)
            if len(ms) <= 100:
                break
        w = 5 * 10**8 if seed % 2 else 10**4
        emit(rng.randint(-w, w), rng.randint(-w, w), ms)
elif size > 12:
    for _ in range(2):
        half = rng.randint(1, min(size, 9800) // 2)
        while True:
            ms = orbit(half, rng.randint(1, 100), zeros=rng.randint(0, 3))
            if len(ms) <= 100:
                break
        w = size // 8
        emit(rng.randint(-w, w), rng.randint(-w, w), ms)
else:
    for _ in range(2):
        half = rng.randint(1, max(1, min(size, 12)))
        ms = orbit(half, rng.randint(1, 5), zeros=rng.randint(0, 2))[:100]
        emit(rng.randint(-20, 20), rng.randint(-20, 20), ms)
