var s = SortedBag<Int>()
print(s.first_ge(3) ?? -1, s.last_lt(5) ?? -1, s.count_range(1, 5))
