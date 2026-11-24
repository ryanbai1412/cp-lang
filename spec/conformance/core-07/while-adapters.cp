// expect: ok, output "3 13"
// rule: 07.7.2
let a = [1, 2, 5, 1, 7]
print(a.take_while(x => x < 3).sum(), a.drop_while(x => x < 3).sum())
