domain B = 0..4
var s = Subset<B>.empty
s.set(B(2), true)
s.set(B(0), true)
print(s.len)
s.set(B(2), false)
print(s.len, yes(s == Subset<B>.single(B(0))))
