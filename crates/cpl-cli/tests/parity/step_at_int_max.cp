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
print("unreached")
