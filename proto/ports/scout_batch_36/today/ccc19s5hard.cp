// CCC '19 S5 (Hard) — Triangle: The Data Structure (today form for proto/cpc)
// original: corpus/submissions/5119931.cpp
let n: 1..=3000 = read()
let k: 1..=n = read()

fn tri(i: Int, j: Int) -> Int: i * (i + 1) / 2 + j

let total = n * (n + 1) / 2
var buf: List<Int> = []
for _ in 0..2 * total: buf.push(0)

for i in 0..n:
    for j in 0..=i:
        let x: Int = read()
        buf[tri(i, j)] = x

var cur_p = 0
var side = 1
while side < k:
    let next_side = min(k, (side * 3 + 1) / 2)
    let d = next_side - side
    let src = cur_p * total
    let dst = (1 - cur_p) * total
    for i in 0..=n - next_side:
        for j in 0..=i:
            let v1 = buf[src + tri(i, j)]
            let v2 = buf[src + tri(i + d, j)]
            let v3 = buf[src + tri(i + d, j + d)]
            buf[dst + tri(i, j)] = max(v1, max(v2, v3))
    cur_p = 1 - cur_p
    side = next_side

let base = cur_p * total
var ans = 0
for i in 0..=n - k:
    for j in 0..=i:
        ans += buf[base + tri(i, j)]
print(ans)
