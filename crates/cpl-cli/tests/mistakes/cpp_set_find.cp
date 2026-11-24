// here: 4
// C++'s `st.find(x) != st.end()` (self-run mistakes): "does not implement `Seq`" and "no method `end`".
let st: Set<Int> = Set()
let found = st.find(3) != st.end()
