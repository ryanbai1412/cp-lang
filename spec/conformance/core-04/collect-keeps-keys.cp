// expect: ok, output "30\n20"
// rule: 04b.1.6
let c = [10, 20, 30, 40][2..].collect()
let (i, j) = (2, 0)
print(c[i])
let d = [10, 20, 30].filter(x => x > 10).collect()
print(d[j])
