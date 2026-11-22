// Text and list helpers from the library: starts_with, ends_with, find, lower, upper,
// insert, remove, and excl_fold.
let s = "Banana"
print(s.starts_with("Ban"), s.starts_with("an"), s.ends_with("ana"), s.ends_with("Bananas"), s.starts_with(""))
print(s.find("nan") ?? -1, s.find("x") ?? -1, s.find("") ?? -1)
print(Str(c.lower for c in s), Str(c.upper for c in s), '7'.upper)
var xs = [1, 2, 3]
xs.insert(0, 9)
xs.insert(4, 8)
xs.insert(2, 7)
print(xs)
print(xs.remove(0), xs.remove(4), xs.remove(1), xs)
for (x, rest) in [2, 3, 5].excl_fold(*): print(x, rest)
let ws = ["a", "b", "c"]
for (x, rest) in ws.excl_fold(Monoid("", (p, q) => p + q)): print(x, rest)
let e: List<Int> = []
print(sum(rest for (_, rest) in e.excl_fold(+)))
