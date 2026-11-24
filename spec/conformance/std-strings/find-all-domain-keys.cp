// expect: ok, output "1 5\n1 5\n1 2"
// rule: 04b.5.2
domain P = 1..=7
let arr: [P]Int = [P] i => [1, 2, 1, 3, 1, 2, 1][i.ord]
print(find_all(arr, [1, 2, 1]).collect())
print(arr.shift_and([1, 2, 1]).collect())
let none: List<Int> = []
print(find_all(arr[P(1)..P(3)], none).collect())
