// expect: ok, output "0 1 1 2 1 2 2 3\n8"
// `Letter.subsets` is a value too: it walks every subset of a fieldless enum in mask order.
enum Letter = A | B | C
var sizes: List<Int> = []
for s in Letter.subsets: sizes.push(s.len)
print(sizes)
print(Letter.subsets.count)
