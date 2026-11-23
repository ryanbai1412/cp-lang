// expect: ok, output "1 4 1"
// rule: 04.2.15
// input: "4\n"
// `Int(v)` of a `1..=n` position is the written value, not the offset; printing a position prints it too (rebuild 7, H-POS-TO-INT, H-POSITION-PRINT).
let n: 1..=10 = read()
domain I = 1..=n
print(Int(I.first!), Int(I.last!), I.first!)
