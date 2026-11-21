let n: Int = read()
var segs: List<(Int, Int)> = [read() for _ in 0..n]
segs.sort()
var cnt = 0
var cur_r = -1_000_000_000_000
for (l, r) in segs:
    if l > cur_r:
        cnt += 1
        cur_r = r
    else:
        cur_r = max(cur_r, r)
print(cnt)
