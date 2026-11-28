// expect: ok, output "8 9 8"
// rule: 12.13.2
// `@aka(pow)` on `Pow`'s `fn pow` makes `a.pow(e)` on its impls E0203 (`lints/pow-method.cp`);
// a struct's own `pow` is its own method and runs.
struct Base:
    b: Int

impl Base:
    fn pow(self, e: Int) -> Int:
        var r = 1
        for _ in 0..e:
            r = r * self.b
        r

print(Base(2).pow(3), 3 ** 2, 2 ** 3)
