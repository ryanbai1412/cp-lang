// expect: E0302 at 3
// rule: std.seq.span
domain I = 0..4
domain J = 0..4
print(span(I(0).l, J(3).r).len)
