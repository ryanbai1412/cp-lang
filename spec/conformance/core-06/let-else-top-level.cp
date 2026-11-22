// expect: ok, output "42"
// rule: R02.4.7
let found: ?Int = some(42)
let v = found else:
    print(-1)
    return
print(v)
