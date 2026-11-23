// expect: ok, output "1 3 1"
// The free form counts where the generator's expression holds (R01.10.1, decision 53);
// on bools the method takes a test (decision 377), and every element is `len`.
let xs = [1, 1, 2, 2]
let all = count(a != b for (a, b) in xs.adjacent)
let bs = [false, true, false]
print(all, bs.len, bs.count(b => b))
