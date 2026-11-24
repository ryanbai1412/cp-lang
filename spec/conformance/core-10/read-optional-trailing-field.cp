// expect: ok, output "5 -1 x"
// rule: 10.2.10
// input: "7 5\nx"
let (n, k): (Int, ?Int) = read()
let s = read<?Str>()
print(k!, read<?Int>() ?? -1, s!)
