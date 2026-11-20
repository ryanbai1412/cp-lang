// Runnable port (proto/cp.py lib-run) of dmopc21c2p4.cp
// original: corpus/submissions/3949962.cpp

let (n, k): (Int, Int) = read()
domain I = 0..n
let a: [I]Int = read()

// Group equal adjacent elements into (val, cnt) runs.
var run_val: List<Int> = []
var run_cnt: List<Int> = []
var idx = 0
while idx < n:
    let v = a[I(idx)]
    let start = idx
    while idx < n and a[I(idx)] == v:
        idx += 1
    run_val.push(v)
    run_cnt.push(idx - start)

let stride = 2 * k + 1
fn blocks(x: Int) -> Int:
    if x <= 0: 0 else: (x + stride - 1) / stride

// States: 0 = Neutral, 1 = Push, 2 = Pull.
domain Carry = 0..3
var dp: [Carry]?Int = [Carry] _ => none
dp[Carry(0)] = 0
var prev_val = 0

for ri in 0..run_val.len:
    let x = run_val[ri]
    let cnt = run_cnt[ri]
    var ndp: [Carry]?Int = [Carry] _ => none

    fn upd(st: Carry, val: Int):
        if let cur = ndp[st]:
            if val < cur: ndp[st] = val
        else:
            ndp[st] = val

    fn relax(base: ?Int, rem: Int):
        if let d = base:
            upd(Carry(0), d + blocks(rem))
            upd(Carry(1), d + blocks(rem + 1))
            upd(Carry(2), d + blocks(rem - k - 1))

    relax(dp[Carry(0)], cnt)
    if prev_val > x:
        relax(dp[Carry(1)], cnt - k - 1)
    else:
        relax(dp[Carry(2)], cnt + 1)

    prev_val = x
    dp = [Carry] s => ndp[s]

print(dp[Carry(0)]!)
