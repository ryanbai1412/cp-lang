// expect: ok, output "2 1"
// rule: R05.3
// Swap by tuple assignment evaluates the right side first.
var a = 1
var b = 2
(a, b) = (b, a)
print(a, b)
