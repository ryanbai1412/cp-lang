// expect: ok, output "6"
// rule: 07.1.3
// A loop over a filtered generator list runs at each kept yield; its `break`
// stops the generator from inside the filter's callback, in both engines and the C++.
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i
fn first_big(n: Int) -> Int:
    var seen = 0
    var k = 0
    while k < 2:
        k += 1
        for x in [y for y in upto(n) if y % 2 == 0]:
            seen += 1
            if x >= 4: break
    seen
print(first_big(10))
