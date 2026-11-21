@!multitest
let n: Int = read()
let a: [0..n]Int = read()
let pos = [x for x in a if x >= 0]
let ok = pos.is_empty or (pos[0] > 0 and all(x == pos[0] for x in pos))
print(if ok: "YES" else: "NO")
