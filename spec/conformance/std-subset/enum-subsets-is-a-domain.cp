// expect: ok, output "12"
// gap: compiler 2026-11-24T04Z
// D.subsets is a domain for any small domain, an enum too.
enum Letter = A | B | C
let best: [Letter.subsets]Int = [Letter.subsets] s => s.len
print(best.sum())
