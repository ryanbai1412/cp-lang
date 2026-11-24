// expect: ok, output "3 5\n5\n3 5"
// each(..) folds positional and named tuples component-wise, and gives a monoid.
print([(1, 5), (2, 3)].fold(each(+, max)))
print([(n: 1, hi: 5), (n: 2, hi: 3)].fold(each(n: +, hi: max)).hi)
print(each(+, max).monoid<(Int, Int)>().op((1, 5), (2, 3)))
