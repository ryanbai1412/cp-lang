// expect: ok, output "3 2 1"
// rule: 08.2.11
// Without `by`, nothing fixes `K`, so it takes `Int` as a literal does (decision 695): the call
// is typed, and `K: Ord` is met by `Int`.
fn first<T, K: Ord>(xs: List<T>, by: ?(T -> K) = None) -> T:
    if let f = by:
        var best = xs[0]
        for x in xs:
            if f(x) < f(best):
                best = x
        return best
    xs[0]
var h = Heap<Int>()
h.push(3)
h.push(1)
print(first([3, 1, 2]), first([3, 1, 2], by: x => (x - 2) * (x - 2)), h.peek!)
