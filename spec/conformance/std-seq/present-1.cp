// expect: ok, output "YES"
print([Some(1), None, Some(3)].present().collect() == [1, 3])
