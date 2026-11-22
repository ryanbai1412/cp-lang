// Codeforces 2135 B — For the Champion (interactive)
//
// Each move query takes at most `step = 10^9`, and all coordinates satisfy
// `|x|, |y| <= 10^9`. Walk in two legs of 4 moves each (only the final
// distance of each leg is needed, because intermediate moves only position us
// strictly outside every anchor's bounding box):
// 1. Move `2 * step` Up (`U, U`) and `2 * step` Right (`R, R`) to
//    `(x0 + 2 * step, y0 + 2 * step)`. Every anchor `(x, y)` lies south-west
//    of this point, so the minimum Manhattan distance `d_ur` equals
//    `(x0 + y0 + 4 * step) - max(x + y)`, giving
//    `ur_pos = x0 + y0 = max(x + y) + d_ur - 4 * step`.
// 2. Move `4 * step` Left (`L, L, L, L`) to `(x0 - 2 * step, y0 + 2 * step)`.
//    Every anchor lies south-east of this point, so `d_ul` equals
//    `(-x0 + y0 + 4 * step) - max(-x + y)`, giving
//    `ul_pos = -x0 + y0 = max(-x + y) + d_ul - 4 * step`.
// Solving recovers `x0 = (ur_pos - ul_pos) / 2` and `y0 = (ur_pos + ul_pos) / 2`.

@!interactive

fn ask(d: Char, dist: Int) -> Int:
    print("?", d, dist)
    read()

@!multitest
let n: 1..=100 = read()
let anchors: [0..n](x: -10 ** 9..=10 ** 9, y: -10 ** 9..=10 ** 9) = read()
let step = 1_000_000_000

for d in "UUR": ask(d, step)
let d_ur = ask('R', step)
for _ in 0..3: ask('L', step)
let d_ul = ask('L', step)

let ur_pos = max(p.x + p.y for p in anchors)! + d_ur - 4 * step
let ul_pos = max(-p.x + p.y for p in anchors)! + d_ul - 4 * step
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)
