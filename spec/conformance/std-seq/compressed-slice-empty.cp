// expect: ok, output "0"
let empty: List<Int> = []
let (domain C, xs) = compress(empty)
print(xs.slice(C.all).len)
