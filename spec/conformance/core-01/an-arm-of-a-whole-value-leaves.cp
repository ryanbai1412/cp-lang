// expect: ok, output "-1 40 -2 8 0 6 99 205"
// rule: 01.7.6
// An arm of the `if`, `match` or block that is a statement's whole value leaves from any depth.
fn first(xs: List<Int>) -> Int:
    let v = if xs.len == 0:
        return -1
    else:
        xs[0]
    v * 10
fn deep(xs: List<Int>) -> Int:
    let v = if xs.len > 0:
        if xs[0] < 0:
            return -2
        xs[0] + 1
    else:
        0
    v
fn blk(n: Int) -> Int:
    let v = {
        if n > 5:
            return 99
        n * 2
    }
    v
var total = 0
for i in 0..6:
    let w = match i % 3:
        0 => continue
        1 => i
        _ => {
            if i == 5:
                break
            i * 100
        }
    total += w
print(first([]), first([4]), deep([-3]), deep([7]), deep([]), blk(3), blk(9), total)
