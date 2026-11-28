// expect: ok, output "2 YES"
// engines: next
let a = [5, 6, 7, 8]
let c = a.cut_after(1)
print(c.ord, yes(c == (1).r))
