// expect: ok, output "2 1 0 2\n0 2\n0 5\n2 7"
// rule: 07.4.7
// `.keys` of a `.rev` walks the source's keys backwards, and keys survive `filter` (R07.3.2) in `.keys` and `.items` alike.
let a = [5, 6, 7]
print([k for k in a.rev.keys], a.rev.keys.first!)
print([k for k in a.filter(x => x != 6).keys])
for (i, x) in a.filter(x => x != 6).items: print(i, x)
