// expect: ok, output "2 0"
// rule: 07.11.4
// `tally` gives a `Counter`, so a key never seen reads 0 (rebuild 7, H-TALLY-TYPE).
let c = [1, 1, 2].tally()
print(c[1], c[7])
