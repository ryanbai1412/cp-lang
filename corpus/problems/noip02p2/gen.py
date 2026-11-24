import sys, random
# seed size: A of length <= min(20, size + 1) over a 2..3-letter alphabet, 0..6 rules of
# length 1..3, and B either A after up to 12 random rewrites (when it stays <= 20) or random;
# --max: A = (ab)^10 with six rules that each match many places and an unreachable B, so a
# depth-10 search explores every rewrite
mx = sys.argv[1] == "--max"
if mx:
    print("ab" * 10, "c")
    for a, b in [("a", "b"), ("b", "a"), ("ab", "ba"), ("ba", "ab"), ("aa", "bb"), ("bb", "aa")]:
        print(a, b)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
al = "abc"[:r.randint(2, 3)]
word = lambda lo, hi: "".join(r.choice(al) for _ in range(r.randint(lo, hi)))
A = word(1, min(20, S + 1))
rules = [(word(1, 3), word(1, 3)) for _ in range(r.randint(0, 6))]
B = word(1, min(20, S + 1))
if rules and r.random() < 0.7:
    cur = A
    for _ in range(r.randint(0, 12)):
        a, b = r.choice(rules)
        at = [i for i in range(len(cur)) if cur.startswith(a, i)]
        if at:
            i = r.choice(at); cur = cur[:i] + b + cur[i + len(a):]
    if 1 <= len(cur) <= 20:
        B = cur
print(A, B)
for a, b in rules:
    print(a, b)
