// expect: ok, output "3"
// input: "101\n"
// `Str(len: ..)` and `of` compose on one type (round 25, H01).
let s: Str(len: 3..=3) of "01" = read()
print(s.len)
