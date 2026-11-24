// expect: ok, output "4\n5\n6\n7\n0 YES\n2\n18446744073709551614\n18446744073709551615\n3\n3 4 5 6\n6 5 4 3\nYES NO\n3 6\n5\n18446744073709551613 18446744073709551614 18446744073709551615"
// rule: 01.8.4
// A `U64` range holds its end wider than its values, so `..= U64(-1)` reaches `U64`'s greatest.
let r = U64(0)..=U64(3)
print(r.len)
for x in U64(5)..U64(8): print(x)
let e = U64(4)..U64(4)
print(e.len, yes(e.is_empty))
let top = U64(-2)..=U64(-1)
print(top.len)
for x in top: print(x)
print((U64(1)..=U64(10)).first_true(x => x >= U64(3))!)
let s = U64(3)..=U64(6)
print(s)
print(s.rev)
print(yes(s.contains(U64(6))), yes(s.contains(U64(7))))
print(s.first!, s.last!)
print(s.nth(2))
print(List(U64(-3)..=U64(-1)))
