// expect: ok, output "3 2 1\n1 4 9"
// rule: 10.3.1
// Any `Seq`, a view or a lazy map included, prints as a sequence.
print([1, 2, 3].rev)
print((1..4).map(x => x * x))
