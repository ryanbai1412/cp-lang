domain D = 0..3
var s = "abc"
var t = s
t.push('d')
var ways: List<Int> = [1, 2]
var next = ways
next.push(5)
let a: [D]Int = [1, 2, 3]
var b = a
b[D(1)] = 9
print(s, t, ways.len, next.len, a[D(1)], b[D(1)], ways[0], next[2])
