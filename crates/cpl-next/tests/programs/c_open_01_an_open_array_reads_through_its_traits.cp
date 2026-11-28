// expect: ok, output "6\n9\n3 3\n1 0\n7"
// rule: R04.8.11
// On `[D]T` with `D` only `Discrete`, `len`, `keys`, `for` and `==` go through their traits,
// so a `List` (`D = Int`) and a domain-keyed array each reach their own impl.
fn f<D: Discrete, T>(a: [D]T) -> Int:
    var t = 0
    for x in a: t += 1
    t + a.len + a.keys.len
print(f<Int, Int>([4, 5]))
print(f([7, 8, 9]))
struct W<D: Discrete, T>:
    n: Int
    k: Int
    pub fn new(a: [D]T) -> W<D, T> = W<D, T>(a.len, a.keys.len)
let w = W<Int, Int>.new([5, 3, 1])
print(w.n, w.k)
fn same<D: Discrete, T: Eq>(a: [D]T, b: [D]T) -> Int = if a == b: 1 else: 0
print(same<Int, Int>([4, 5], [4, 5]), same<Int, Int>([4], [5]))
domain V = 0..3
fn count<D: Domain, T>(a: [D]T) -> Int = a.len + a.keys.len + D.count - 2
print(count([V] v => Int(v)))
