let n: Int = read()
var total = 0
for d in 1..=n:
    var hits = 0
    var k = d
    for m in d..=n by k:
        k += 1
        hits += 1
    total += hits
print(total)
for i in 0..10 by 3: print(i)
for i in 3..3 by 2: print("never", i)
for i in 0..=9 by 3:
    if i == 3: continue
    if i == 9: break
    print("kept", i)
var last = -1
for i in 5..=7 by 100: last = i
print(last)
for i in (Int.MAX - 5)..=Int.MAX by 4: print(Int.MAX - i)
var sum = 0
for a in 0..n by 2:
    for b in a..n by a + 1: sum += b
print(sum)
