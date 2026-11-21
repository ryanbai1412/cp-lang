fn twice<R>(body: () -> R) -> R:
    let r = body()
    r

var n = 3
twice(() => print(n))
let k = twice(() => n * 10)
print(n, k)
