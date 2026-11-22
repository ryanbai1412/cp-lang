// expect: ok, output "3 3 1"
let xs = [1, 1, 2, 2]
let all = count(a != b for (a, b) in xs.adjacent)
let bs = [false, true, false]
print(all, bs.count(), bs.count(b => b))
