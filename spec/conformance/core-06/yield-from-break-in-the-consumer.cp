// expect: ok, output "1\n2\nstopped"
// rule: 06.3.6
// A consumer's `break` stops the walk inside the delegated generator, as it does in
// the loop `yield from` stands for; the loop's `else` doesn't run.
fn upto(n: Int) -> yields Int:
    if n > 0:
        yield from upto(n - 1)
        yield n
for x in upto(100):
    if x == 3: break
    print(x)
else:
    print("never")
print("stopped")
