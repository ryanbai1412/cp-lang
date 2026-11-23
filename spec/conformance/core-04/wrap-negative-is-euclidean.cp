// expect: ok, output "4 3 2"
// rule: 04.2.13
// `V.wrap` reduces Euclidean-style, so a negative integer wraps from the end.
domain P = 0..5
print(Int(P.wrap(-1)), Int(P.wrap(-7)), Int(P.wrap(12)))
