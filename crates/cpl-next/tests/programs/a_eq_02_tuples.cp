// expect: ok, output "YES NO\nYES NO YES\nYES YES"
// rule: R08.1.6
// Tuples compare field by field through core's ordinary impls; `Ord` is lexicographic.
let p = (1, 'b')
print(yes(p == (1, 'b')), yes(p == (1, 'c')))
print(yes((1, 2, 3) < (1, 3, 0)), yes((2, 0) < (1, 9)), yes((1, 2, 3, 4) <= (1, 2, 3, 4)))
print(yes((0, (1, 2)) > (0, (1, 1))), yes((5, 5) != (5, 6)))
