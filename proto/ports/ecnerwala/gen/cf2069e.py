#!/usr/bin/env python3
import random
import sys


def main():
    seed = int(sys.argv[1])
    size = int(sys.argv[2])
    rng = random.Random(seed)
    t = rng.randint(1, max(1, min(8, size)))
    print(t)
    for _ in range(t):
        n = rng.randint(1, max(1, size))
        mode = rng.randrange(4)
        if mode == 0:
            s = "".join(rng.choice("AB") for _ in range(n))
        elif mode == 1:
            # Mostly alternating with occasional breaks
            chars = [rng.choice("AB")]
            for _ in range(n - 1):
                if rng.random() < 0.8:
                    chars.append("B" if chars[-1] == "A" else "A")
                else:
                    chars.append(chars[-1])
            s = "".join(chars)
        elif mode == 2:
            # Concatenated AB and BA blocks
            parts = []
            while sum(len(p) for p in parts) < n:
                k = rng.randint(1, max(1, min(6, n)))
                tok = rng.choice(["AB", "BA", "A", "B"])
                parts.append(tok * k)
            s = "".join(parts)[:n]
        else:
            s = rng.choice("AB") * n
        cnt_a = s.count("A")
        cnt_b = n - cnt_a
        # Mix tight and loose capacities
        if rng.random() < 0.7:
            a = rng.randint(0, cnt_a)
            b = rng.randint(0, cnt_b)
            need = max(0, cnt_a - a, cnt_b - b)
            ab = rng.randint(0, max(0, need + 2))
            ba = rng.randint(0, max(0, need + 2))
        else:
            a = rng.randint(0, n)
            b = rng.randint(0, n)
            ab = rng.randint(0, n)
            ba = rng.randint(0, n)
        print(s)
        print(a, b, ab, ba)


if __name__ == "__main__":
    main()
