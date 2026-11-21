// Codeforces 2135 B (interactive): walk 2*step up-right then 4*step left to
// recover x + y and -x + y from the farthest Manhattan anchors.
fn ask(d: Char, dist: Int) -> Int:
    print("?", d, dist)
    flush()
    read()

@!multitest
let n: Int = read()
let anchors: [0..n](Int, Int) = read()
let step = 1_000_000_000

for d in "UUR": ask(d, step)
let d_ur = ask('R', step)
for _ in 0..3: ask('L', step)
let d_ul = ask('L', step)

let ur_pos = max(x + y for (x, y) in anchors)! + d_ur - 4 * step
let ul_pos = max(-x + y for (x, y) in anchors)! + d_ul - 4 * step
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)
flush()
