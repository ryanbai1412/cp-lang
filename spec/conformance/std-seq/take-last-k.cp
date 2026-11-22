// expect: ok, output "2 3\n0"
// rule: 07.7
print([1, 2, 3].take_last(2).collect())
let empty: List<Int> = []
print(empty.take_last(3).count())
