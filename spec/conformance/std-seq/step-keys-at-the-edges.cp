// expect: ok, output "3 YES NO YES NO 16 10 13 16"
// rule: std.seq.Stepped.keys
// A step view keeps the source keys on its stride: the first and last keys on it are in, the ones between and past the end are out.
let a = [10, 11, 12, 13, 14, 15, 16]
let s = a.step(3)
print(s.len, yes(3 in s.keys), yes(4 in s.keys), yes(6 in s.keys), yes(7 in s.keys), s[6], [x for x in s])
