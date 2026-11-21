var one: [1..=2, 1..=3]Int = 7
let k: Int = read()
one[1, 2..=k] += 1
print(one[1, ..])
one[1, 3..=k + 1] += 1
print("unreached")
