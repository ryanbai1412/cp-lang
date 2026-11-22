// expect: ok, output "6"
// rule: 01.6.2
let opt: ?Int = 5
let x = opt else:
    print(yes(false))
    return
print(x + 1)
