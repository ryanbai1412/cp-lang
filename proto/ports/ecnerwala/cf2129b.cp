@! multitest
let n: Int = read()
let p: [0..n]Int = read()
print(
    sum(min(count(p[j] > p[i] for j in 0..i), count(p[j] > p[i] for j in i + 1..n)) for i in 0..n)
)
