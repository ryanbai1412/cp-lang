// expect: ok, output "2 0\n3 10\n4 20"
// rule: 04b.1.7 04b.5.3
fn tens<S: Indexed<K, T>, K, T>(s: S) -> [K]Int:
    var z = List.filled(s.len, 0)
    for i in 0..s.len: z[i] = i * 10
    z.rekey(s.keys)
let a = [5, 6, 7, 8, 9]
for (k, x) in tens(a[2..5]).items: print(k, x)
