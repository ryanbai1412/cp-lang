fn scale<const M: Int>(x: Int) -> Int = x % M
let x = read<Int>()
print(scale::<998244353>(x))
