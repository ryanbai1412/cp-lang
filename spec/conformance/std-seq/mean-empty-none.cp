// expect: ok, output "YES YES"
let empty: List<Int> = []
print(yes(empty.mean() == None), yes([1, 2].mean() == Some(1.5)))
