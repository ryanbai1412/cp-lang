"""nccc8s1 generator: an alphabet (distinct lowercase letters), then a word over it.
--max: all 26 letters, |s| = 100."""
import random
import sys

big = sys.argv[1] == "--max"
r = random.Random(0 if big else int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 8
abc = "abcdefghijklmnopqrstuvwxyz"
al = abc if big else "".join(sorted(r.sample(abc, r.randint(1, min(26, size + 2)))))
n = 100 if big else r.randint(1, min(100, size + 1))
print(al)
print("".join(r.choice(al) for _ in range(n)))
