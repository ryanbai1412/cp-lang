// expect: ok, output "YES"
print([1, 2, 3].filter_map(x => if x != 2: Some(x * 10) else: None).collect() == [10, 30])
