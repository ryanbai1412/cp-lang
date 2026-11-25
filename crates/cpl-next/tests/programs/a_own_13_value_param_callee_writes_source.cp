// expect: ok, output "2"
// rule: R05 R4/R10; conformance core-05/param-unchanged-by-a-callee-writing-its-source
// Value parameter is a lend; a callee writing the global source: caller's `a` unchanged during call (R4/R10) -- reading both is defined.
var g = [1]
fn f(a: List<Int>) -> Int:
    g.push(5)
    a.len + g.len - 1
print(f(g))
