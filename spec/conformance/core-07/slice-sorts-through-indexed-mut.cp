// expect: ok, output "5 1 3 4 2\n3 1 5 4 2\nb c a"
// rule: 07.4.17
// A slice of a `var` source is `IndexedMut`, so `sort` and `reverse` change the source's elements in place through it.
var a = [5, 3, 1, 4, 2]
a[1..4].sort()
print(a)
a[0..3].reverse()
print(a)
var s = ["c", "a", "b"]
s[1..].reverse()
s[..2].reverse()
print(s)
