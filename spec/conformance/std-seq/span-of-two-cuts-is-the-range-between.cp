// expect: ok, output "4 1 3\n0\n3 2 1"
// rule: std.seq.span
domain D = 0..6
let r = span(D(1).l, D(4).r)
print(r.len, r.first!, Int(r.last!) - 1)
print(span(D(2).l, D(2).l).len)
print([span(i.l, D(5).r).len for i in D(3)..])
