let n: Int = read()
var segs: List<(Int, Int)> = [read() for _ in 0..n]
segs.sort()
var total = 0
var cur_l = 0
var cur_r = -1_000_000_000_000
for (l, r) in segs:
    if l > cur_r:
        if cur_r > -1_000_000_000_000:
            total += cur_r - cur_l
        cur_l = l
        cur_r = r
    else:
        cur_r = max(cur_r, r)
if cur_r > -1_000_000_000_000:
    total += cur_r - cur_l
print(total)
