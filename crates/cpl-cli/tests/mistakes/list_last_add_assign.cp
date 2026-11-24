// here: 4
// `a.last += 5`: "`+=` can't be applied to an ?Int", with no fix.
var a = [1, 2, 3]
a.last += 5
print(a)
