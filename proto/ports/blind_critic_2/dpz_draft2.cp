// DP Contest Z (Frog 3): stones at increasing heights, a jump i -> j costs (h[j] - h[i])^2 + c.
let n: 2..=2e5 = read()
let c: 1..=1e12 = read()
domain I = 1..=n

fn climb() -> [I]1..=1e6:
    var x = 0
    [I] i => { x += rand(1, 3); x }

let h: [I]1..=1e6 = read(gen: climb) where all(h[i] < h[(i + 1)!] for i in I if i != I.last)

// dp[i] = h[i]^2 + c + min over j of (-2 h[j]) * h[i] + (dp[j] + h[j]^2): lines with
// decreasing slope queried at increasing x, so a monotone queue of lines.
@solution
fn main():
    var dp: [I]Int = 0
    var lines: List<(m: Int, b: Int)> = []
    var head = 0
    fn bad(p: (m: Int, b: Int), q: (m: Int, b: Int), r: (m: Int, b: Int)) -> Bool:
        // q is never the minimum once r arrives
        (r.b - p.b) * (q.m - p.m) <= (q.b - p.b) * (r.m - p.m)
    for i in I:
        if i != I.first:
            while head + 1 < lines.len and lines[head].m * h[i] + lines[head].b >= lines[head + 1].m * h[i] + lines[head + 1].b:
                head += 1
            dp[i] = lines[head].m * h[i] + lines[head].b + h[i] ** 2 + c
        let line = (m: -2 * h[i], b: dp[i] + h[i] ** 2)
        while lines.len - head >= 2 and bad(lines[lines.len - 2], lines[lines.len - 1], line):
            lines.pop()
        lines.push(line)
    print(dp[I.last])

@brute(main)
fn slow():
    var dp: [I]Int = 0
    for i in I:
        if i != I.first:
            dp[i] = min(dp[j] + (h[i] - h[j]) ** 2 + c for j in I.first..i)!
    print(dp[I.last])
