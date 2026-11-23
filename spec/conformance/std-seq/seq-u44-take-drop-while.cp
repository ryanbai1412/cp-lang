// expect: ok, output "1 2 5 1 1\n1 2 5 1"
// Acceptance for seq/U44 (prefixes and suffixes by count or predicate), handbook/product/std/sequences-usecases.md.
let a = [1, 2, 5, 1]
print(List(a.take(2)), List(a.drop(2)), List(a.take_last(1)))
print(a.take_while(x => x < 3).collect(), a.skip_while(x => x < 3).collect())
