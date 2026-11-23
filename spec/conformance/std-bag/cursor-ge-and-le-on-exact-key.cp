// expect: ok, output "5 5 YES YES"
let b = SortedBag.of([2, 5])
print(b.key_of(b.cursor_ge(5)!), b.key_of(b.cursor_le(5)!), yes(b.cursor_gt(5) == None), yes(b.cursor_lt(2) == None))
