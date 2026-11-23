// Pairs i < j with a[i] + a[j] == k, written cold for the Milestone 1 loop run, with a planted
// bug: the inner loop starts at i, so an element pairs with itself.
// class: off-by-one

let (n, k): (1..=8, 0..=20) = read()
let a: [0..n]0..=10 = read()
var count = 0
for i in 0..n:
    for j in i..n:  // planted
        if a[i] + a[j] == k: count += 1
print(count)

@alt(program)
fn by_value():
    var seen: [0..=10]Int = 0
    var answer = 0
    for i in 0..n:
        if k - a[i] >= 0 and k - a[i] <= 10: answer += seen[k - a[i]]
        seen[a[i]] += 1
    print(answer)
