// expect: ok, output "0 0 0 0 none\n2 0 none 2 0\n2 2 none 0 0\n2 4 none\n7 3 0 -5 -5 0"
// input: "5\n"
// rule: 02.4.6
// `min` and `max` of optionals: `None` is the identity on either side, packed or not.
let n: 1..=10 = read()
domain I = 0..n
let none: ?I = None
let zero: ?I = Some(I.first!)
let two: ?I = Some(I.first!.next!.next!)
print(min(none, zero), min(zero, none), max(none, zero), max(zero, none), min(none, none))
print(max(zero, two), min(two, zero), max(none, none), max(none, zero, two), min(two, none, zero))
let cells = [none, two, none, zero]
print(
    cells.fold(max),
    [none, two].fold(min),
    [none, none].fold(max),
    [zero, none].fold(min),
    SegTree<I, ?I>(min).all ?? zero,
)
var at = SegTree<I, ?I>(max)
at.set(I.first!, two)
at.set(I.last!, Some(I.last!))
print(at.fold(I.first!..I.last!), at.all, SegTree<I, ?I>(max).all)
let big: ?Int = Some(7)
let neg: ?Int = Some(-5)
let gone: ?Int = None
print(
    max(gone, big, neg) ?? 0,
    max(neg, 3),
    min(gone, gone) ?? 0,
    min(neg, gone) ?? 0,
    min(big, neg, gone) ?? 0,
    max(gone, gone) ?? 0,
)
