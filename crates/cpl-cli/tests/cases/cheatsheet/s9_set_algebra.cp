var s = Set<Int>()
s.add(1)
print((s | s).len, (s & s).len, (s - s).len, (s ^ s).len)
