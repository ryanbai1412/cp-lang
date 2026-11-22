// expect: ok, output "YES"
print(yes([Some(1), None, Some(3)].present().collect() == [1, 3]))
