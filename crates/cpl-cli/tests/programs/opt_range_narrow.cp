// `?lo..hi` that fits 31 bits is stored in 4 bytes and still range-checked.
fn layers(n: Int) -> List<?(0..2_147_483_648)>:
    var level: List<?(0..2_147_483_648)> = List.filled(n, None)
    level[0] = 0
    for i in 1..n:
        if i % 3 != 0 and level[i - 1] != None: level[i] = level[i - 1]! + 1
    return level
let n: Int = read()
var level = layers(n)
level[1] = None
print(level[0] ?? -1, level[1] ?? -1, level[2] ?? -1, level[3] ?? -1)
var lv: List<?(0..100)> = [None, None, None]
lv[0] = 7
lv[2] = n
print(lv[0] ?? -1, lv[1] ?? -1, lv[2] ?? -1)
