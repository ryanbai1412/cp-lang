// expect: E0303
// `T(x)` where `T`'s `new` takes nothing converts `x` by its `Into<T>` impl; with none, `x` is one argument too many.
struct Tally:
    n: Int

    fn new() -> Tally = Tally(0)

let t = Tally([1, 2]) // <- here
