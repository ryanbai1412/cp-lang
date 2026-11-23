// expect: ok, output "YES"
print(yes(iterate(Some(1), x => if x < 8: Some(2 * x) else: None).collect() == [1, 2, 4, 8]))
