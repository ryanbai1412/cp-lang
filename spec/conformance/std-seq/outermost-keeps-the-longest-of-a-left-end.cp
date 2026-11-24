// expect: ok, output "YES YES\nYES YES"
// rule: 07.9.4
// `outermost` sorts the ranges themselves by left end and keeps them in place: of ranges sharing
// a left end the longest stays, wherever it comes; equal ranges keep one, and a range inside
// an earlier kept one, of any left end, goes.
print(yes([3..5, 0..4, 0..9, 10..12, 0..9, 2..9].outermost() == [0..9, 10..12]), yes([3..9, 2..7, 3..4, 2..3, 4..9].outermost() == [2..7, 3..9]))
let e: List<Range<Int>> = []
print(yes(e.outermost() == []), yes([5..6].outermost() == [5..6]))
