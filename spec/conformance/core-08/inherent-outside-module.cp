// expect: E0501 at 3
// rule: 08.1.3
mod m:
    pub struct P: x: Int
impl m.P:
    fn inc(self) -> Int = self.x + 1
