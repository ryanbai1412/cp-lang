// expect: ok, output "c"
// rule: 10.2.3
// input: "2 3\nabc\ndef"
let (h, w) = read<(Int, Int)>()
domain R = 0..h
domain C = 0..w
let g = read<[R, C]Char>()
print(g[R(0), C(2)])
