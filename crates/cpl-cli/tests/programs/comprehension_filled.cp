let n: 0..=100 = read()
var b = [List<Int>() for _ in 0..2 * n]
var c = [0 for _ in 3..n]
var d = [(0, "x") for _ in 5..2]
b[0].push(3)
b[1].push(4)
c[0] += 1
print(b.len, b[0].len, b[1], b[2].len, c.len, c[0], c[1], d.len)
