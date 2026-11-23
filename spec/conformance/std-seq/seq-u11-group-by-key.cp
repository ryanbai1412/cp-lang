// expect: ok, output "0 9\n1 7 4\n2 5 2"
// Acceptance for seq/U11 (group by a key, in a defined order), handbook/product/std/sequences-usecases.md.
for (k, xs) in [5, 2, 7, 4, 9].group(by: x => x % 3):
    print(k, xs)
