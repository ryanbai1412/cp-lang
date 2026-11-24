// expect: ok, output "3 5\n5\n3 5"
// each(..) folds positional and named tuples component-wise, and gives a monoid.
print(fold(each(+, max), [(1, 5), (2, 3)]))
print(fold(each(n: +, hi: max), [(n: 1, hi: 5), (n: 2, hi: 3)]).hi)
print(each(+, max).monoid<(Int, Int)>().op((1, 5), (2, 3)))
