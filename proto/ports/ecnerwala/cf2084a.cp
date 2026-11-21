@!multitest
let n: Int = read()
if let pos = first(i for i in 1..=n if (n + 1) % i == 0 and (n + 1) % (i + 1) == 0):
    print([if j == pos: n elif j < pos: j else: j - 1 for j in 1..=n])
else:
    print(-1)
