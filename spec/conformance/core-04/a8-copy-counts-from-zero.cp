// expect: ok, output "e\n3"
// rule: 04a.2.8 04a.11.6 04b.1.6
let s = "hello"
let t = Str(s[1..4])
let i = 0
print(t[i])
let w = List([1, 2, 3, 4][2..])
print(w[i])
