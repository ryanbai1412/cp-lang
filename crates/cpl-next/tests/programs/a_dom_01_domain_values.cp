// expect: ok, output "1 4 4\n4 3 2 1\n2 3"
// rule: R04.2.21
// A domain named as a value is its whole range: its ends, its walk and `rev` give positions.
domain V = 1..=4
print(V.first!, V.last!, V.count)
print(V.rev)
for v in V.rev:
    if Int(v) == 2: print(v, Int(v) + 1)
