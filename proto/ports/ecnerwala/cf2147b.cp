@!multitest
let n: Int = read()
print([i for i in (1..n).rev] + [n] + [i for i in 1..n] + [n])
