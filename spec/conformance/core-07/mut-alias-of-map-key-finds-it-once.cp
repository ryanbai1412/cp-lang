// expect: ok, output "1\n1\n0\n2\n5"
// rule: 07.12.9
// `let p = mut m[k]` names one entry: `op=` through it starts an absent key at the default of `V`,
// reads see every write through it, and `=` inserts or replaces, as `m[k]` itself does.
var m = HashMap<Int, Int>()
for i in 0..5:
    let p = mut m[i % 3]
    p += 1
    let gone = p == 2
    if gone: m.remove(i % 3)
print(m.len)
print(m[2])
var t = Map<Str, Int>()
let q = mut t["a"]
q = 2
print(t.len - 1)
print(t["a"])
let r = mut t["a"]
r max= 5
print(t["a"])
