// Above `@!multitest` the program is still single-test: a `return` there ends everything.
let mode: 0..=1 = read()
if mode == 0:
    print("off")
    return
@!multitest
let n: 1..=9 = read()
print(n)
