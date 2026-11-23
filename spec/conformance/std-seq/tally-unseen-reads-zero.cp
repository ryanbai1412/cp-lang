// expect: ok, output "2 1 0\n1 4 0"
// `tally` returns a Counter, so a key never seen reads 0.
let c = "abca".tally()
print(c['a'], c['b'], c['z'])
let d = [3, 1, 4, 1, 5].tally(by: x => x % 2)
print(d[0], d[1], d[7])
