// expect: ok, output "1 2\n2"
// rule: 05.3.5
// A field pushed to and then set whole still has its old value under `old`: the pushes land
// in a list nothing reads before the store replaces it, so `old` takes the old list, no copy.
struct S:
    var ks: List<Int>
fn grow(mut s: S):
    let old = s.ks
    for i in 0..2: s.ks.push(i)
    s.ks = [old.len]
    print(old)
var s = S(ks: [1, 2])
grow(mut s)
print(s.ks)
