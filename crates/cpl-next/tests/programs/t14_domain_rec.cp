// expect: ok, output "1 1 2\n2 2 4\n3 3 6\n101\n202\n303"
// input: "3\n"
// A new domain per recursive call, captured by a closure that outlives the inner call.
// A global slot per domain would give the wrong size here.
fn total<D>(a: [D]Int) -> Int:
    var s = 0
    for v in D: s += a[v]
    s
fn go(n: Int) -> List<Int -> Int>:
    var out: List<Int -> Int> = []
    if n == 0: return out
    domain V = 0..n
    let a: [V]Int = 1
    out = go(n - 1)
    // V here is still the outer one, of size n.
    var b: [V]Int = 2
    print("{n} {V.count} {total(b)}")
    // `a` is a heap value, so a closure would borrow it (R05.10.1); capture its sum instead.
    let ta = total(a)
    out.push(x => x + V.count * 100 + ta)
    out
let n: Int = read()
let fs = go(n)
for i in 0..fs.len: print(fs[i](0))
