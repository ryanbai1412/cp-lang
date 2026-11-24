// `(lo..hi).rev by step` walks lo, lo + step, ... backward, on the vm:
// from the last value in range down to `lo`, with the bounds read once.
let n: Int = read()
var lo = 1
var seen: List<Int> = []
for i in (lo..=n).rev by 4:
    lo += 100
    seen.push(i)
print(seen, lo)
for i in (0..n).rev by 3: print(i)
for i in (n..n).rev by 2: print("never", i)
for i in (n..=n).rev by 5: print("once", i)
let big = Int.MAX - 2
for i in (big..=Int.MAX).rev by 2: print(i - big)
let low = Int.MIN
for i in (low..low + 5).rev by 2: print(i - low)
var total = 0
for i in (0..n * 10).rev by 7:
    if i % 2 == 0: continue
    if i < 20: break
    total += i
print(total)
