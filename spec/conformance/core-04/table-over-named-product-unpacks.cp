// expect: ok, output "0 1 2 3 10 11 12 13\n12"
// rule: 01.8.2 01.8.1
// A table over a named product domain takes a k-parameter lambda that unpacks the key, as `[A, Dir] (i, d) => e` does.
enum Dir = N | E | S | W
domain A = 0..2
domain P = (A, Dir)
let t: [P]Int = [P] (i, d) => Int(i) * 10 + Int(d)
print(t)
print(t[1, Dir.S])
