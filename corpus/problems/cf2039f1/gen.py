import random
import sys

if sys.argv[1] == "--max":
    print(1)
    print(100_000)
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
ms = [r.randint(1, max(1, S * r.choice([1, 10, 100]))) for _ in range(r.randint(1, 5))]
print(len(ms))
print("\n".join(map(str, ms)))
