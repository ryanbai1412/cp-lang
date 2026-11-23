// `x!` on `none` stops the built program too: status 3, not a wrong value.
let k: Int = read()
let o: ?Int = if k > 5: None else: k
print("before")
print(o!)
print("never printed")
