// expect: ok, output "YES"
print(yes(SortedBag.of([1, 4, 6]).first_true(x => x >= 3) == Some(4)))
