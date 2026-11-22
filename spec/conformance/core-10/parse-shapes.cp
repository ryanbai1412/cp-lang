// expect: ok, output "6 4 x 6 x"
// rule: 10.2.3 10.2.4
let a = "3 1 2".parse<[3]Int>()!
let (k, s) = "4 x".parse<(Int, Str)>()!
let m = "-1".parse<Mod<7>>()!
let c = "  x".parse<Char>()!
print(a.sum(), k, s, m, c)
