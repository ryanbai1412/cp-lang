@!multitest
let n: Int = read()
var ans: [0..(2 * n)]Int = 0
var cur: List<Int> = []
var oth: List<Int> = []
for i in 0..(2 * n):
    cur.push(i + 1)
    print("?", cur.len, cur)
    flush()
    let res: Int = read()
    if res != 0:
        ans[i] = res
        let _ = cur.pop()
        oth.push(i + 1)
for x in cur:
    oth.push(x)
    print("?", oth.len, oth)
    flush()
    let res: Int = read()
    ans[x - 1] = res
    let _ = oth.pop()
print("!", ans)
flush()
