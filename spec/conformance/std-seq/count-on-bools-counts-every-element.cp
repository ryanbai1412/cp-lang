// expect: ok, output "1 3 1"
// The free form counts where the generator's expression holds (R01.10.1, decision 53);
// the method with no predicate counts every element.
let xs = [1, 1, 2, 2]
let all = count(a != b for (a, b) in xs.adjacent)
let bs = [false, true, false]
print(all, bs.count(), bs.count(b => b))
