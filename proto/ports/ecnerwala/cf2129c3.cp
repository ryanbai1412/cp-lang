fn ask(inds: List<Int>) -> Int:
    print("?", inds.len, inds)
    flush()
    read()

fn build_seq() -> List<Int>:
    var seq: List<Int> = []
    var s = 0
    var tot = 0
    var i = 1
    while true:
        if i * (i + 1) / 2 > s:
            tot += 2 * i + 1
            s += i * (i + 1) / 2
            if tot > 1000: break
            seq.push(i)
        i += 1
    seq

let seq = build_seq()
let blk = seq.len

@!multitest
let n: Int = read()
var ans: [0..n]Char = '?'
var mi = 0
var ma = n
while ma - mi > 1:
    let md = (mi + ma) / 2
    if ask([i + 1 for i in mi..=md]) > 0:
        ma = md
    else:
        mi = md
let open_i = mi + 1

var i = 0
while i < n:
    let inds = [min(i + z, n - 1) for z in 0..blk]
    var cnd: List<Int> = []
    for z in 0..blk:
        for _ in 0..seq[z]:
            cnd.push(open_i)
            cnd.push(inds[z] + 1)
        cnd.push(open_i)
    var r = ask(cnd)
    for z in (0..blk).rev():
        let cnt = seq[z] * (seq[z] + 1) / 2
        if r >= cnt:
            ans[inds[z]] = ')'
            r -= cnt
        else:
            ans[inds[z]] = '('
    i += blk
print("!", Str(ans))
flush()
