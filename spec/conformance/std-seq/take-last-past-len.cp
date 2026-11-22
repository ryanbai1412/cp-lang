// expect: ok, output "1 2 3\n0"
print([1, 2, 3].take_last(5).collect())
print([1, 2, 3].take_last(0).count())
