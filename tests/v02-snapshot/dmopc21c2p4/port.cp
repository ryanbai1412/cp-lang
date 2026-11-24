// Source: DMOPC 2021 Contest 2 P4 (Water Mechanics), https://dmoj.ca/problem/dmopc21c2p4
// Problem: N cells in a row with heights h_i <= 1e9 and an integer 1 <= K <= N <= 1e6. Water
//   poured into a cell spreads at most K cells each way over equal heights, the limit resets
//   whenever it flows down, and it never flows up. Print the fewest pours that wet every cell.
// Solution: group equal heights into runs; a run of length L alone needs ceil(L / (2K + 1))
//   pours. DP left to right over runs with a state for the boundary to the next run:
//   neutral, this run's water spills right into the lower next run (it then gets K + 1
//   cells free), or the higher next run's water spills left into this one (covering its
//   last K + 1 cells). O(N).
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
    if x <= 0: 0 else: ceil_div(x, stride)

// States: 0 = Neutral, 1 = Push, 2 = Pull.
domain Carry = 0..3
var dp: [Carry]?Int = [Carry] _ => None
dp[Carry(0)] = 0
var prev_val = 0

for ri in 0..run_val.len:
    let x = run_val[ri]
    let cnt = run_cnt[ri]
    var ndp: [Carry]?Int = [Carry] _ => None

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
