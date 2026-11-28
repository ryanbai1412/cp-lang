// expect: ok, output "0 1"
// engines: next
domain P = 0..4
domain E = [3]
fn bit(x: Bool) -> Int = if x: 1 else: 0
print(bit(Cut<P>.counted), bit(E.counted))
