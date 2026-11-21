// Codeforces 2147 A: minimum steps to reach (x, y) with alternating strictly increasing moves.
@!multitest
let (x, y): (Int, Int) = read()
print(if x < y: 2 elif x - y <= 1 or y == 1: -1 else: 3)
