// expect: ok, output "0 5 0 5 5"
// rule: 01.3
// `lazy` is not reserved (the lazy array was removed): it names an array and is indexed like any other.
var lazy: [0..3]Int = 0
lazy[1] = 5
var pending: List<Int> = []
pending.push(lazy[1])
print(lazy[0], lazy[1], lazy[2], lazy.sum(), pending)
