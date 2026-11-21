fn ask(inds: List<Int>) -> Int:
    print("?", inds.len, inds)
    flush()
    read()

fn build_seq() -> List<Int>:
    var res: List<Int> = []
    var (s, tot, k) = (0, 0, 1)
    while true:
        let tri = k * (k + 1) / 2
        if tri > s:
            tot += 2 * k + 1
            s += tri
            if tot > 1000: break
            res.push(k)
        k += 1
    res

let seq = build_seq()
let blk = seq.len

@!multitest
let n: Int = read()
var (mi, ma) = (0, n)
while ma - mi > 1:
    let md = (mi + ma) / 2
    if ask([idx + 1 for idx in mi..=md]) > 0: ma = md
    else: mi = md
let open_i = mi + 1

var ans: [0..n]Char = '?'
for start in 0..n by blk:
    let inds = [min(start + z, n - 1) for z in 0..blk]
    var cnd: List<Int> = []
    for z in 0..blk:
        for _ in 0..seq[z]:
            cnd.push(open_i)
            cnd.push(inds[z] + 1)
        cnd.push(open_i)

    var rem = ask(cnd)
    for z in (0..blk).rev:
        let cnt = seq[z] * (seq[z] + 1) / 2
        if rem >= cnt:
            ans[inds[z]] = ')'
            rem -= cnt
        else:
            ans[inds[z]] = '('

print("!", Str(ans))
flush()
