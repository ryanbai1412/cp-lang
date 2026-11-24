// here: 5
// A bare `Int` index on a scan reads as a cut or a position: E0301, fixed to `pre.cut(k)`.
let k: Int = read()
let pre = scan([3, 1, 4], +)
print(pre[k])
