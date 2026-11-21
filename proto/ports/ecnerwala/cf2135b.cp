fn ask(d: Char, j: Int) -> Int:
    print("?", d, j)
    flush()
    read()

@!multitest
let n: Int = read()
let anchors: List<(Int, Int)> = [read::<(Int, Int)>() for _ in 0..n]
let j = 1_000_000_000
let _ = ask('U', j)
let _ = ask('U', j)
let _ = ask('R', j)
let d_ur = ask('R', j)
let _ = ask('L', j)
let _ = ask('L', j)
let _ = ask('L', j)
let d_ul = ask('L', j)
var max_ur = -2 * j
var max_ul = -2 * j
for (x, y) in anchors:
    max_ur max= x + y
    max_ul max= -x + y
let ur_pos = max_ur + d_ur - 4 * j
let ul_pos = max_ul + d_ul - 4 * j
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)
flush()
