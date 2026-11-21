"""Writes the judge-size inputs for the std.dp benchmarks: python3 gen.py DIR."""
import random
import sys

out = sys.argv[1] if len(sys.argv) > 1 else "."
r = random.Random(1)
n = 200000
h = sorted(r.sample(range(1, 10**6 + 1), n))
open(f"{out}/frog3.in", "w").write(f"{n} {10**12}\n" + " ".join(map(str, h)) + "\n")
open(f"{out}/dnc.in", "w").write(f"{n} 20\n" + " ".join(str(r.randint(0, 1000)) for _ in range(n)) + "\n")
n = 3000
open(f"{out}/knuth.in", "w").write(f"{n}\n" + " ".join(str(r.randint(1, 10**9)) for _ in range(n)) + "\n")
n = 5000000
open(f"{out}/mono.in", "w").write(f"{n} 1000\n" + " ".join(str(r.randint(0, 10**9)) for _ in range(n)) + "\n")
n = 1000000
open(f"{out}/monge.in", "w").write(f"{n} {10**6}\n" + " ".join(str(r.randint(0, 1000)) for _ in range(n)) + "\n")
n = 1000000
open(f"{out}/aliens.in", "w").write(f"{n} 5000\n" + " ".join(str(r.randint(-10**6, 10**6)) for _ in range(n)) + "\n")
