// expect: ok, output "4 2"
// engines: next
domain P = 0..4
print(Cut<P>.wrap(-1).ord, Cut<P>.wrap(7).ord)
