// `read<T>()` on a lowercase fn takes type arguments (decision 28.4).
let n = read<Int>()
let (a, b) = read<(Int, Int)>()
let k = 2
print(n, a + b, n < k, k > (n))
