// Codeforces 2022 C, tuned: orig.cpp's dp over the 2n/3 cut points, pushed forward, so
// each step scores only the shapes that start at that cut (port.cp scores every column).
@!multitest
let n: 3..=100_000 = read()
let top: Str = read()
let bottom: Str = read()
let v = [[Int(ch == 'A') for ch in top], [Int(ch == 'A') for ch in bottom]]
fn won(a: Int, b: Int, c: Int) -> Int = Int(a + b + c >= 2)
let d = 2 * n / 3
var dp = [List.filled(d + 1, 0), List.filled(d + 1, 0)]
for cur in 0..d:
    let x = cur * 3 / 2
    if cur % 2 == 1:
        for z in 0..2:
            let o = 1 - z
            dp[0][cur + 1] = max(dp[0][cur + 1], dp[z][cur] + won(v[z][x], v[z][x + 1], v[o][x + 1]))
            if cur + 2 <= d:
                let bars = won(v[z][x], v[z][x + 1], v[z][x + 2]) + won(v[o][x + 1], v[o][x + 2], v[o][x + 3])
                dp[z][cur + 2] = max(dp[z][cur + 2], dp[z][cur] + bars)
    else:
        let flat = won(v[0][x], v[0][x + 1], v[0][x + 2]) + won(v[1][x], v[1][x + 1], v[1][x + 2])
        dp[0][cur + 2] = max(dp[0][cur + 2], dp[0][cur] + flat)
        dp[0][cur + 1] = max(dp[0][cur + 1], dp[0][cur] + won(v[0][x], v[1][x], v[1][x + 1]))
        dp[1][cur + 1] = max(dp[1][cur + 1], dp[0][cur] + won(v[0][x], v[1][x], v[0][x + 1]))
print(dp[0][d])
