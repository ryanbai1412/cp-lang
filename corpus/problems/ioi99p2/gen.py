import sys, random, string
# usage: gen.py seed size; size >= 1e6 gives a 1e6-letter text and 100 words
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
L = string.ascii_letters
if S < 1000000:
    al = L[:r.randint(1, 4)]
    words = ["".join(r.choice(al) for _ in range(r.randint(1, 4))) for _ in range(r.randint(1, 5))]
    text = "".join(r.choice(al) for _ in range(r.randint(1, max(1, S))))
    if r.random() < 0.5:  # long gaps of filler around the 1000-letter span limit
        text = "".join(c + "z" * r.choice([0, 0, 300, 499, 500, 998, 999]) for c in text)[:1000000]
elif r.random() < 0.5:  # time: 100 words of 100 letters, a random text
    words = ["".join(r.choice(L) for _ in range(100)) for _ in range(100)]
    text = "".join(r.choice(L) for _ in range(1000000))
else:  # planted words spread to spans of exactly 1000 or 1001 letters
    up = string.ascii_uppercase
    words = ["".join(r.sample(up, r.randint(2, 5))) for _ in range(100)]
    t = []
    while len(t) < 990000:
        w = r.choice(words); span = r.choice([1000, 1001, r.randint(len(w), 1000)])
        gaps = sorted(r.sample(range(1, span - 1), len(w) - 2)) if len(w) > 2 else []
        piece = ["z"] * span; piece[0] = w[0]; piece[-1] = w[-1]
        for g, c in zip(gaps, w[1:-1]): piece[g] = c
        t += piece + ["y"] * r.randint(0, 3000)
    text = "".join(t)[:1000000]
print(len(words)); print("\n".join(words)); print(text)
