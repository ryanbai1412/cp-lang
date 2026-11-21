fn ask(d: Char, dist: Int) -> Int:
    print("?", d, dist)
    flush()
    read()

@! multitest
let n: Int = read()
let anchors: [0..n](Int, Int) = read()
let step = 1_000_000_000

ask('U', step)
ask('U', step)
ask('R', step)
let d_ur = ask('R', step)
ask('L', step)
ask('L', step)
ask('L', step)
let d_ul = ask('L', step)

let max_ur = max(x + y for (x, y) in anchors)!
let max_ul = max(-x + y for (x, y) in anchors)!
let ur_pos = max_ur + d_ur - 4 * step
let ul_pos = max_ul + d_ul - 4 * step
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)
flush()
