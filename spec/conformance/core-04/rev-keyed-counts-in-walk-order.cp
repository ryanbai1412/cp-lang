// expect: ok, output "0 4\n1 3\n2 2\n3 1\n3 4"
// rule: 04a.9.6
// `.keyed().items` pairs each element with a counter from 0 in walk order, also after
// `.rev`; `.rev.items` keeps the source keys.
let a = [1, 2, 3, 4]
for (i, x) in a.rev.keyed().items: print(i, x)
for (i, x) in a.rev.items.take(1): print(i, x)
