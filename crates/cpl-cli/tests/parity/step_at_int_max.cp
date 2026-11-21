let big = Int.MAX - 12
for i in (big..Int.MAX) by 5: print(i - big)
for i in big..Int.MAX by 4: print(i - big)
for i in (big..Int.MAX).rev by 5: print(i - big)
print([i - big for i in (big..Int.MAX) by 6])
let lo = Int.MIN + 3
for i in (lo..Int.MAX) by Int.MAX: print(i - lo)
var hits = 0
for i in (0..100) by 7:
    if i % 2 == 0: continue
    hits += i
print(hits)
for i in (big..=Int.MAX) by 5: print(i - big)
print("reached")
for i in (Int.MIN..Int.MAX) by Int.MAX: print(i < 0)
let h = Int.MAX
for i in h - 1..=h: print(h - i)
for i in (h - 1..=h) by 1: print(h - i)
for i in (h - 1..=h).rev: print(h - i)
for i in (h - 6..=h).rev by 3: print(h - i)
for i in 3..=2: print("none")
let m = Int.MIN
for i in (m..m + 2).rev: print(i - m)
print([h - i for i in h - 2..=h], sum(i for i in 1..=10))
var k = h
k -= 2
print(k + 2 in h - 1..=h, k in h - 1..=h, 3 in 1..=3, 4 in 1..=3, 3 in 1..3)
