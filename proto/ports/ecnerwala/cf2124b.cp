@!multitest
let n: Int = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
print(a[0] + min(a[0], a[1]))
