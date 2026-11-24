import sys, random
# seed size: n <= min(20, size + 1) words, up to 7 of length 1..8 over a 2..4-letter alphabet, more as
# x z..z y links between n/2..n letters, then a start letter some word begins with; --max: n = 20
# words x z..z y over ten letters, each leading to one of the next three letters, so the depth-first
# search walks many long chains (each word twice)
if sys.argv[1] == "--max":
    r = random.Random(12); L = "abcdefghij"
    ws = [L[i % 10] + "z" * r.randint(2, 6) + L[r.choice([(i + 1) % 10, (i + 2) % 10, (i + 3) % 10])] for i in range(20)]
    print(20); print(*ws, sep="\n"); print("a"); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(20, S + 1))
if n > 7:  # dense chains over few letters blow up the search: link each word to 1..3 next letters
    c = r.randint(n // 2, n)
    ws = [chr(97 + i % c) + "z" * r.randint(0, 4) + chr(97 + (i % c + r.randint(1, 3)) % c) for i in range(n)]
    print(n); print(*ws, sep="\n"); print("a"); sys.exit()
al = "abcd"[:r.randint(2, 4)]
hi = r.randint(1, 8)
ws = ["".join(r.choice(al) for _ in range(r.randint(1, hi))) for _ in range(n)]
print(n); print(*ws, sep="\n"); print(r.choice(ws)[0])
