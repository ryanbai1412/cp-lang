// expect: ok, output "6 0"
// (spec) R4: a value argument the call also writes is copied in (conformance core-05/param-unchanged-by-a-callee-writing-its-source).
// self is the global; the method writes the global by name while holding self by value.
struct S:
    xs: List<Int>
    fn sum_then_clear(self) -> Int:
        var t = 0
        for x in xs:
            t += x
            g = S([])
        t
var g = S([1, 2, 3])
print(g.sum_then_clear(), g.xs.len)
