// error: a tuple is a value, not a place to assign to
var a = 1
var b = 2
(a, b) = (b, a + b)
