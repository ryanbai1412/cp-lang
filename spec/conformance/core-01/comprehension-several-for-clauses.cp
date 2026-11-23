// expect: ok, output "3\neoe"
// rule: 01.8.2
let n = 3
let ps = [(i, j) for i in 0..n, j in i + 1..n]
print(ps.len)
print([if i % 2 == 0: 'e' else: 'o' for i in 0..3])
