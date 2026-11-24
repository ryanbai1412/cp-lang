// expect: E0501
// rule: 08.5.2
impl Hash for F64: // <- here
    fn hash(self) -> Int = 0
