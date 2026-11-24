// expect: ok, output "1 0 0 1 3\n50"
// rule: R04b.1.10 R09.3.10
// A step view's `keys` are the source keys the stride visits; `in` tests one in O(1), and `at` reads only those.
let a = [10, 20, 30, 40, 50]
let s = a.step(2)
print(Int(2 in s.keys), Int(3 in s.keys), Int(5 in s.keys), Int(0 in s.keys), s.keys.len)
print(s.at(4))
