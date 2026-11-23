let hub: ?Int = None
let two: ?Int = Some(2)
let n = 3
if n >= 2 and hub is None:
    print(1)
if n >= 2 and two is Some(v) and v == 2:
    print(2)
if n < 2 and hub is None:
    print(3)
if n >= 2 and two is None:
    print(4)
