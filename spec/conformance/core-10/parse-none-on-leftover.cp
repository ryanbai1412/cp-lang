// expect: ok, output "-1 -1 3 1"
// rule: 10.2.4
let s = "10"
print("5 6".parse<Int>() ?? -1, "x".parse<Int>() ?? -1, s.parse<Mod<7>>()!, s.parse<Mod<3>>()!)
