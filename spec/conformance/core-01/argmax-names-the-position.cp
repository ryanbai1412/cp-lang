// expect: ok, output "1\n2 1\n1 5"
// rule: 01.10.1
domain R = 0..3
let t: [R]Int = [R] r => (r.ord * 2) % 3
let m: ?R = argmax(t[r] for r in R)
print(m!.ord)
print(argmax(i * j for i in 0..3, j in 0..2)!)
print(argmax(w for (k, w) in [(0, 3), (1, 5), (2, 4)])!)
