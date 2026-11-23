enum Side:
    Left
    Mid
    Right

var dp: [0..3]Int = [1, 2, 3]
for i in 0..5:
    var nxt: [0..3]Int = [dp[1] + i, dp[2] + 1, dp[0] + 2]
    dp = nxt

var flags: [4]Bool = false
flags[1] = true
flags[3] = true

var side_cnt: [Side]Int = 0
side_cnt[Side.Left] = 10
side_cnt[Side.Mid] = 20
side_cnt[Side.Right] = 30

var fwd = 0
for x in [dp[0], dp[1], dp[2]]:
    fwd = fwd * 100 + x
var rev = 0
for x in [1, 2, 3].rev:
    rev = rev * 10 + x

var scratch: [0..3]Int = 0
scratch.fill(7)
scratch[0] = 9
scratch.swap(0, 2)
scratch.reverse()
assert scratch.len == 3

fn sum3(xs: [3]Int) -> Int = xs[0] + xs[1] + xs[2]

var copy: [0..3]Int = scratch.clone()
assert copy == scratch
assert sum3([dp[0], dp[1], dp[2]]) == dp[0] + dp[1] + dp[2]

print(dp[0], dp[1], dp[2], fwd, rev, yes(flags[1]), yes(flags[2]), side_cnt[Side.Mid], scratch[0], scratch[2])
