// expect: ok, output "YES"
print(yes([1, 2, 5, 1].drop_while(x => x < 3).collect() == [5, 1]))
