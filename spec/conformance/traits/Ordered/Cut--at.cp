// expect: ok, output "4 1"
// engines: next
domain P = 0..4
print(Cut<P>.at(4)!.ord, (Cut<P>.at(9) ?? Cut<P>.nth(1)).ord)
