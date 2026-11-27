// expect: ok, output "NO 7 2\n3 8\n1 5\nYES -1\n4 0"
// rule: R02.6.6
// A list grows and shrinks at its end: pop, truncate, clear, is_empty.
var xs = [5, 6, 7]
print(yes(xs.is_empty), xs.pop() ?? 0, xs.len)
xs.push(8)
xs.truncate(9)
print(xs.len, xs[2])
xs.truncate(1)
print(xs.len, xs[0])
xs.clear()
print(yes(xs.is_empty), xs.pop() ?? -1)
xs.push(4)
print(xs.pop() ?? -1, xs.len)
