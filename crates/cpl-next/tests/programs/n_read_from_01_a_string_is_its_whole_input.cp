// expect: ok, output "7 4 x YES YES YES 8\n2 YES\nYES YES 3 YES YES"
// rule: R10.2.4
// input: "7 8\n"
// `read<T>(from: s)` reads with `s` as the whole input, by the same `Read` impls, and leaves
// standard input where it was: `None` for a bad, missing or leftover token.
let a = read<Int>()
let p = read<(Int, Str)>(from: " 4 x\n")!
let left = read<Int>(from: "4 5") == None
let bad = read<Int>(from: "x") == None
let missing = read<(Int, Int)>(from: "4") == None
print(a, p, yes(left), yes(bad), yes(missing), read<Int>())
let xs = read<List<Int>>(from: "2 1 2")!
print(xs.len, yes("12".parse<Int>() == Some(12)))
let outside = read<1..=3>(from: "9") == None
let part = read<(Int, 0..=5)>(from: "1 6") == None
print(yes(outside), yes(part), read<1..=3>(from: "3")!, yes("300".parse<0..=255>() == None), yes(read<Char>(from: " ") == None))
