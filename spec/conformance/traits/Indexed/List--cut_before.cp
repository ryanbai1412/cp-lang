// expect: ok, output "1 YES"
// engines: next
let a = [5, 6, 7, 8]
let c = a.cut_before(1)
print(c.ord, yes(c == (1).l))
