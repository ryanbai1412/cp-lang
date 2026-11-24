// expect: ok, output "7 4 x YES YES 8"
// input: "7 8\n"
// rule: 10.2.4
// `read<T>(from: s)` reads with `s` as the whole input and leaves standard input where it was.
let a = read<Int>()
let p = read<(Int, Str)>(from: " 4 x\n")!
print(a, p, yes(read<Int>(from: "4 5") == None), yes(read<1..=3>(from: "9") == None), read<Int>())
