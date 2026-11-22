// expect: ok, output "6"
// rule: 07.12.1
print([1, 2, 5, 1].drop_while(x => x < 3).sum())
