// `and`, `or` and `not` in a condition jump straight to their branch on the
// VM, keeping short-circuit order and side effects.
fn hit(tag: Int, v: Bool) -> Bool:
    print("hit", tag)
    v

fn main():
    let n: Int = read()
    let a = n > 2
    let b = n % 2 == 0
    let o: ?Int = if b: Some(n) else: None
    if not (a and b) or (n < 0 and hit(1, true)):
        print("one")
    if hit(2, a) and hit(3, not b) or hit(4, n == 5):
        print("two")
    if not (hit(5, b) or hit(6, a)):
        print("three")
    elif o != None and not (o! > 10):
        print("four")
    else:
        print("five")
    var i = 0
    while i < n and not (i == 3 and b):
        i += 1
    var j = n
    while not (j <= 0 or j == 2):
        j -= 1
    let c = if a and not b: 10 else: 20
    print(i, j, c, yes(a and b), yes(a or not b), yes(not (a or b)))

main()
