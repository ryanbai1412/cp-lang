// A closure reading a non-`mut` SegTree parameter calls `get`, which pushes
// pending tags down: the C++ holds the parameter by value, as a direct read does.
domain H = 1..=8
fn first_le(st: SegTree<H, Int, Int>, target: Int) -> ?Int:
    (1..9).first_true(mid => st.get(H(mid)) <= target)
var st = SegTree<H, Int, Int>(min, act: add, a: [H] _ => 0)
st.apply(H(1)..=H(5), 3)
print(first_le(st, 0) ?? -1)
