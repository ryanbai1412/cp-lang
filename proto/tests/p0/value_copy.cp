var s = "abc"
var t = s
t.push('d')
var ways: List<Int> = [1, 2]
var next = ways
next.push(5)
let a: [0..3]Int = [1, 2, 3]
var b = a
b[1] = 9
print(s, t, ways.len, next.len, a[1], b[1], ways[0], next[2])
