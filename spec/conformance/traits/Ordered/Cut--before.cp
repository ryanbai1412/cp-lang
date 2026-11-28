// expect: ok, output "1"
// engines: next
domain P = 0..4
print(Cut<P>.nth(1).before.len)
