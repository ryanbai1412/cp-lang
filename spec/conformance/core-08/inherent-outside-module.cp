// expect: E0501
// rule: 08.1.3
mod m:
    pub struct P: x: Int
impl m.P: // <- here
    fn inc(self) -> Int = self.x + 1
