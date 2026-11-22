// expect: ok, output "3 2 2\nYES 0"
print([1, 2, 3].pairs().count(), [1, 2, 3, 4].count(x => x % 2 == 0), [3, 4].map(x => x).count())
print(yes([10, 20, 30].pairs().collect() == [(10, 20), (10, 30), (20, 30)]), [5].pairs().count())
