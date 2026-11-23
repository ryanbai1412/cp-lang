// A lambda given for an optional fn parameter infers its key type.
fn pick<T, K>(xs: List<T>, by: ?(T -> K) = None) -> T:
    var best = xs[0]
    if let f = by:
        for x in xs:
            if f(x) < f(best): best = x
    return best

var h: Heap<Int> = Heap(by: x => desc(x))
h.push(1)
h.push(5)
h.push(3)
print(h.pop()!, h.pop()!)
print(pick([3, 1, 4], by: x => [-x]))
