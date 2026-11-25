// expect: ok, output "2"
// input: "5\n"
fn count_evens(xs: List<Int>) -> Int:
    var cnt = 0
    let bump = (x: Int) => cnt += 1
    for x in xs:
        if x % 2 == 0: bump(x)
    cnt
print(count_evens([1, 2, 3, 4]))
