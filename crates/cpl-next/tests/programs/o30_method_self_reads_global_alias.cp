// expect: E0602
// (spec) R10 (callee writes `g` by name while `g` is lent as self; reviewer and today's compiler accept: QUESTION)
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
print(g.sum_then_clear(), g.xs.len)  // <- here
