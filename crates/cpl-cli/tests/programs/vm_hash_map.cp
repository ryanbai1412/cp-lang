let n: Int = read()
var cnt: HashMap<Int, Int> = []
var rows: HashMap<Int, List<Int>> = []
for i in 0..n:
    cnt[i % 5] += i
    cnt[i % 3] = cnt[i % 3] + 1
    if not rows.contains(i % 4): rows[i % 4] = [i, i * i]
var hit = 0
for k in 0..7:
    if cnt.contains(k): hit += cnt[k]
    match cnt.get(k):
        Some(v): print(k, v)
        None: print(k, "none")
cnt.remove(0)
print(hit, cnt.len, rows.len, rows[1], rows[3][1])
