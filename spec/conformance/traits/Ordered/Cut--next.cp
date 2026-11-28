// expect: ok, output "2 0"
// engines: next
domain P = 0..4
print(Cut<P>.nth(1).next!.ord, (Cut<P>.nth(4).next ?? Cut<P>.nth(0)).ord)
