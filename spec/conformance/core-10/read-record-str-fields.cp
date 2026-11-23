// expect: ok, output "cd 3"
// rule: 10.2.1
// input: "2\nab 3\ncd 4\n"
// A record reads its fields in order, one token each, `Str` fields too (rebuild round 17 H10).
let n = read<1..=5>()
let rs = read<[n](name: Str, w: Int)>()
print(rs[1].name, rs[0].w)
