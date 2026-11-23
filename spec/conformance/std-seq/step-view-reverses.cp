// expect: ok, output "4 2 0 3 1"
// rule: R04a.9.3
// A step view is a sequence like any other, so `.rev` walks it backwards and a comprehension collects it.
let p = [0, 1, 2, 3, 4]
let r = [x for x in p.even.rev] + [x for x in p.odd.rev]
print(r)
