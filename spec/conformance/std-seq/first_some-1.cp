// expect: ok, output "YES"
print([1, 4, 9].first_some(x => if x > 3: Some(x * 10) else: None) == Some(40))
