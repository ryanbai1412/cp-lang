// expect: E0203 at 2
var s = SortedBag<Int>()
print(s.first_ge(3), s.last_lt(5), s.count_range(1, 5))
