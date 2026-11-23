// expect: ok, output "YES"
print(yes(SortedBag.of([1, 4, 6]).last_true(x => x <= 5) == Some(4)))
