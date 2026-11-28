// expect: ok, output "3"
// engines: next
domain P = 0..4
print(Cut<P>.nth(1).after.len)
