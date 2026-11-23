struct Road:
    to: Int
impl Road:
    fn twice(self) -> Int = self.to * 2
fn far(r: Road) -> Int = r.twice()
print( far(Road(to: 3)) )
