// expect: ok, output "1\n20 30"
// rule: R04.3.1
// A key written as an `if` or a `match` takes its type from its branches: positions of the
// array's domain index it as positions, integers index a list as integers.
domain V = 0..4
let par: [V]?V = [V] v => Some(v)
let dep: [V]Int = [V] v => Int(v.ord)
let (x, y) = (V(1), V(2))
print(par[if dep[y] < dep[x]: y else: x]!)
let a = [10, 20, 30]
let c = true
print(a[if c: 1 else: 2], a[match 0:
    0 => 2
    _ => 0
])
