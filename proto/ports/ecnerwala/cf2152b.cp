@!multitest
let (n, rk, ck, rd, cd): (Int, Int, Int, Int, Int) = read()
let dx = if rd > rk: rd elif rd < rk: n - rd else: 0
let dy = if cd > ck: cd elif cd < ck: n - cd else: 0
print(max(dx, dy))
