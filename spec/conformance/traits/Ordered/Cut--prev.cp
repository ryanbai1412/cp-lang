// expect: ok, output "2"
// engines: next
domain P = 0..4
print(Cut<P>.nth(3).prev!.ord)
