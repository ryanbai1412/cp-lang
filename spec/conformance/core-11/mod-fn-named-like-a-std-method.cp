// expect: ok, output "3 9"
// rule: 11.1.2
// A module's fn is called by its name even when std has a method of that name.
mod stats:
    pub fn tally(xs: List<Int>) -> Int = xs.len
    pub fn sum(a: Int, b: Int) -> Int = a + b
print(stats.tally([4, 8, 9]), stats.sum(4, 5))
