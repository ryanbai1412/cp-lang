var calls = 0
fn note(x: Int) -> Int:
    calls += 1
    x
fn pick(xs: List<Int>, by: ?(Int -> Int) = None) -> Int:
    var best = xs[0]
    for x in xs:
        let k = by
        if let f = k:
            if f(x) > f(best):
                best = x
        elif x > best:
            best = x
    best
fn both(xs: List<Int>, cmp: ?((Int, Int) -> Bool)) -> Bool:
    let less = (a: Int, b: Int) => if let c = cmp: c(a, b) else: a < b
    less(note(xs[0]), note(xs[1]))
fn top(xs: List<Int>, key: ?(Int -> Int) = None) -> Int:
    if key != None:
        return key!(xs[0])
    xs[note(1)]
let k = read::<Int>()
print(pick([3, 9, 4]), pick([3, 9, 4], by: x => -x * k))
print(yes(both([1, 2], (a, b) => a > b + k)), yes(both([5, 2], (a, b) => a * k < b)), calls)
print(top([4, 7]), top([6, 8]), calls)
