// The longest strictly increasing run of adjacent elements, written cold for the Milestone 1 loop
// run, with a planted bug: the best is taken once after the loop, so only the last run counts.
// class: boundaries

let n: 1..=8 = read()
let a: [0..n]1..=5 = read()
var best = 1
var len = 1
for i in 1..n:
    if a[i] > a[i - 1]: len += 1
    else: len = 1
best max= len
print(best)

@alt(program)
fn every_start():
    var answer = 0
    for i in 0..n:
        var j = i + 1
        while j < n and a[j] > a[j - 1]: j += 1
        answer max= j - i
    print(answer)
