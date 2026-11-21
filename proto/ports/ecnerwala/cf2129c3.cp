// Codeforces 2129 C3 (interactive): binary-search an open bracket, then decode
// blocks of brackets in one query each using superincreasing triangular weights.
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
            if tot > 1000: break
            s += tri
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
    for (z, k) in seq.items:
        for _ in 0..k: cnd.extend([open_i, inds[z] + 1])
        cnd.push(open_i)

    var rem = ask(cnd)
    for (z, k) in seq.items.rev:
        let cnt = k * (k + 1) / 2
        if rem >= cnt:
            ans[inds[z]] = ')'
            rem -= cnt
        else:
            ans[inds[z]] = '('

print("!", ans)
flush()
