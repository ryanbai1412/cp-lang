// expect: ok, output "12"
// rule: contract 4 regions
// return from inside two nested generator bodies inside a fn.
fn up(n: Int) -> yields Int:
    for i in 0..n: yield i
fn find() -> Int:
    for a in up(5):
        for b in up(5):
            if a * b == 2: return a * 10 + b
    0 - 1
print(find())
