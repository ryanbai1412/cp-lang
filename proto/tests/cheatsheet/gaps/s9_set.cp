var s = Set<Int>()
s.add(3); s.add(1); s.add(7); s.remove(3)
print(1 in s, s.first, s.last, s.first_ge(2), s.first_gt(7), s.last_le(6), s.last_lt(1), s.len, s.is_empty)
print(s.pop_first(), s.pop_last(), s.len)
