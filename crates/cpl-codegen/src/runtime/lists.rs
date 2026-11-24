//! Lists, arrays and grids: building, indexing, first and last, sorting
//! and the views over them.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
    Piece {
        name: "swap_at",
        needs: &[],
        code: "\
// `xs.swap(i, j)`: both inside the sequence, or the program stops.
template <class S> inline void swap_at(S& s, ll i, ll j) {
    unsigned long long n = s.size();
    if (__builtin_expect((unsigned long long)i >= n || (unsigned long long)j >= n, 0)) { cpl::flush(); fprintf(stderr, \"swap(%lld, %lld) is outside a list of length %llu\\n\", i, j, n); exit(3); }
    std::swap(s[i], s[j]);
}
",
    },
    Piece {
        name: "truncate",
        needs: &[],
        code: "\
// `xs.truncate(n)`: keep the first n, all of them when there are fewer. A negative n stops the program.
template <class S> inline void truncate(S& s, ll n) {
    if (__builtin_expect(n < 0, 0)) { cpl::flush(); fprintf(stderr, \"truncate to negative length %lld\\n\", n); exit(3); }
    if ((unsigned long long)n < s.size()) s.resize((size_t)n);
}
",
    },
    Piece {
        name: "list_of",
        needs: &[],
        code: "\
// `[a, b]` of lists: the elements are built in order, as in `{a, b}`, then
// moved, not copied, into the list.
template <class T, size_t N> std::vector<T> list_of(T (&&a)[N]) {
    return std::vector<T>(std::make_move_iterator(a), std::make_move_iterator(a + N));
}
",
    },
    Piece {
        name: "reserve",
        needs: &[],
        code: "\
// `xs.reserve(n)`: reserve capacity for at least n elements (a sorted map has none). A negative n stops the program.
template <class S> inline auto reserve_room(S& s, size_t n, int) -> decltype(s.reserve(n), void()) { s.reserve(n); }
template <class S> inline void reserve_room(S&, size_t, long) {}
template <class S> inline void reserve(S& s, ll n) {
    if (__builtin_expect(n < 0, 0)) { cpl::flush(); fprintf(stderr, \"reserve to negative capacity %lld\\n\", n); exit(3); }
    reserve_room(s, (size_t)n, 0);
}
",
    },
    Piece {
        name: "have",
        needs: &["fail"],
        code: "\
template <class O> decltype(auto) have(O& o) {
    if (!o.has_value()) fail(\"`!` on `None`: there is no value here\");
    return *o;
}
template <class O, std::enable_if_t<!std::is_lvalue_reference<O>::value, int> = 0> auto have(O&& o) {
    if (!o.has_value()) fail(\"`!` on `None`: there is no value here\");
    return std::move(*o);
}
",
    },
    Piece {
        name: "grid",
        needs: &["fail"],
        code: "\
// An array with K dimensions, stored flat in row-major order.
template <class T, int K> struct Grid {
    std::vector<T> d;
    ll n[K] = {};
    template <class... I> size_t at(I... i) const { ll ix[K] = {(ll)i...}; ll p = 0; for (int k = 0; k < K; k++) p = p * n[k] + ix[k]; return (size_t)p; }
    template <class... I> T& operator()(I... i) { return d[at(i...)]; }
    template <class... I> const T& operator()(I... i) const { return d[at(i...)]; }
    // Indexed by an `Int` that nothing has proved in range.
    template <class... I> size_t in(I... i) const { ll ix[K] = {(ll)i...}, p = 0; bool bad = 0; for (int k = 0; k < K; k++) bad |= (unsigned long long)ix[k] >= (unsigned long long)n[k], p = p * n[k] + ix[k]; if (bad) fail(\"index outside the array\"); return p; }
    template <class... I> T& ck(I... i) { return d[in(i...)]; }
    template <class... I> const T& ck(I... i) const { return d[in(i...)]; }
    bool operator==(const Grid& o) const { return d == o.d; }
    bool operator<(const Grid& o) const { return d < o.d; }
};
template <class T, int K> Grid<T, K> grid(std::array<ll, K> n, const T& fill) {
    Grid<T, K> g; ll cells = 1;
    for (int k = 0; k < K; k++) { g.n[k] = n[k]; cells *= n[k]; }
    g.d.assign((size_t)cells, fill);
    return g;
}
",
    },
    Piece {
        name: "pop",
        needs: &[],
        code: "\
template <class T> std::optional<T> pop(std::vector<T>& v) { if (v.empty()) return std::nullopt; T x = std::move(v.back()); v.pop_back(); return x; }
inline std::optional<u8> pop(std::string& s) { if (s.empty()) return std::nullopt; u8 x = (u8)s.back(); s.pop_back(); return x; }
",
    },
    Piece {
        name: "first",
        needs: &[],
        code: "\
template <class C> auto first(const C& v) { using T = std::decay_t<decltype(*std::begin(v))>; return v.empty() ? std::optional<T>() : std::optional<T>(*std::begin(v)); }
inline std::optional<u8> first(const std::string& s) { return s.empty() ? std::optional<u8>() : std::optional<u8>((u8)s.front()); }
",
    },
    Piece {
        name: "last",
        needs: &[],
        code: "\
template <class C> auto last(const C& v) { using T = std::decay_t<decltype(*std::begin(v))>; return v.empty() ? std::optional<T>() : std::optional<T>(*std::prev(std::end(v))); }
inline std::optional<u8> last(const std::string& s) { return s.empty() ? std::optional<u8>() : std::optional<u8>((u8)s.back()); }
",
    },
    Piece {
        name: "has",
        needs: &[],
        code: "\
template <class T, class U> bool has(const std::vector<T>& v, const U& x) { return std::find(v.begin(), v.end(), x) != v.end(); }
",
    },
    Piece {
        name: "sorted",
        needs: &[],
        code: "\
template <class It, class V, class L> void merge_sort(It a, size_t n, std::vector<V>& buf, L& lt) { if (n <= 1) return; size_t m = n / 2, i = 0, j = m, k = 0; merge_sort(a + m, n - m, buf, lt); merge_sort(a, m, buf, lt); while (i < m && j < n) buf[k++] = lt(a[j], a[i]) ? std::move(a[j++]) : std::move(a[i++]); while (i < m) buf[k++] = std::move(a[i++]); std::move(buf.begin(), buf.begin() + k, a); }
// The interpreter's merge sort, so `cpl profile` counts the same comparisons natively.
template <class It, class L> void stable_sort_lt(It b, It e, L lt) { using V = typename std::iterator_traits<It>::value_type; std::vector<V> buf(b, e); merge_sort(b, (size_t)(e - b), buf, lt); }
template <class It, class L = std::less<>> void sort_seq(It b, It e, L lt = L()) { using V = typename std::iterator_traits<It>::value_type; if constexpr (std::is_integral_v<V>) std::sort(b, e, lt); else stable_sort_lt(b, e, lt); }
template <class T> std::vector<T> sorted(std::vector<T> v) { sort_seq(v.begin(), v.end()); return v; }
template <class T> std::vector<T> sorted(const std::set<T>& s) { return std::vector<T>(s.begin(), s.end()); }
",
    },
    Piece {
        name: "reversed",
        needs: &[],
        code: "\
template <class T> std::vector<T> reversed(std::vector<T> v) { std::reverse(v.begin(), v.end()); return v; }
",
    },
    Piece {
        name: "row",
        needs: &["fail"],
        code: "\
// A row of exactly K cells held inside a struct. It is made from a list
// of K cells and is a list again to whatever takes it whole.
template <class T, int K> struct Row {
    std::array<T, K> a{};
    Row() = default;
    static Row filled(const T& x) { Row r; r.a.fill(x); return r; }
    Row(const std::vector<T>& v) {
        if (v.size() != (size_t)K) fail(\"a row of another length than its field has\");
        std::copy(v.begin(), v.end(), a.begin());
    }
    std::vector<T> vec() const { return std::vector<T>(a.begin(), a.end()); }
    operator std::vector<T>() const { return vec(); }
    size_t size() const { return (size_t)K; }
    auto begin() const { return a.begin(); }
    auto end() const { return a.end(); }
    auto rbegin() const { return a.rbegin(); }
    auto rend() const { return a.rend(); }
    T& operator[](size_t i) { return a[i]; }
    const T& operator[](size_t i) const { return a[i]; }
    bool operator==(const Row& o) const { return a == o.a; }
    bool operator!=(const Row& o) const { return a != o.a; }
    bool operator<(const Row& o) const { return a < o.a; }
};
template <class T, int K> T& at(Row<T, K>& v, ll i) { if ((unsigned long long)i >= (unsigned long long)K) fail(\"index outside the list\"); return v.a[(size_t)i]; }
template <class T, int K> const T& at(const Row<T, K>& v, ll i) { if ((unsigned long long)i >= (unsigned long long)K) fail(\"index outside the list\"); return v.a[(size_t)i]; }
",
    },
    Piece {
        name: "at",
        needs: &["fail"],
        code: "\
// A list is indexed by an `Int`, which nothing has proved in range.
template <class T> T& at(std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> const T& at(const std::vector<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
inline char& at(std::string& s, ll i) { if ((unsigned long long)i >= s.size()) fail(\"index outside the string\"); return s[(size_t)i]; }
inline const char& at(const std::string& s, ll i) { if ((unsigned long long)i >= s.size()) fail(\"index outside the string\"); return s[(size_t)i]; }
// `V(x)` and `V.at(i)`: an ordinal that must be one of the domain's `len`.
inline int pos(ll ord, ll len) { if ((unsigned long long)ord >= (unsigned long long)len) fail(\"outside the domain\"); return (int)ord; }
inline ll pos_wide(ll ord, ll len) { if ((unsigned long long)ord >= (unsigned long long)len) fail(\"outside the domain\"); return ord; }
",
    },
    Piece {
        name: "at_array",
        needs: &["fail"],
        code: "\
template <class T, size_t K> T& at(std::array<T, K>& v, ll i) { if ((unsigned long long)i >= K) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T, size_t K> const T& at(const std::array<T, K>& v, ll i) { if ((unsigned long long)i >= K) fail(\"index outside the list\"); return v[(size_t)i]; }
",
    },
    Piece {
        name: "filled_array",
        needs: &[],
        code: "\
template <class T, size_t K> inline std::array<T, K> filled_array(const T& x) { std::array<T, K> a; a.fill(x); return a; }
",
    },
    Piece {
        name: "sort_by",
        needs: &["sorted"],
        code: "\
// `sort(by: key)`: each key is worked out once, in order, before any is
// compared; equal keys keep their order.
// A `()` key (a `by:` left at `_ => ()`) makes every key equal.
template <class S, class F> void sort_by(S& seq, F&& key, bool desc) {
    using K = std::decay_t<decltype(key(*seq.begin()))>;
    using T = typename S::value_type;
    if constexpr (std::is_void_v<K>) {
        for (const auto& x : seq) key(x);
        (void)desc;
        return;
    } else {
    std::vector<K> keys;
    keys.reserve(seq.size());
    for (const auto& x : seq) keys.push_back(key(x));
    std::vector<size_t> order(keys.size());
    for (size_t i = 0; i < order.size(); i++) order[i] = i;
    stable_sort_lt(order.begin(), order.end(), [&](size_t a, size_t b) { return desc ? keys[b] < keys[a] : keys[a] < keys[b]; });
    std::vector<T> sorted;
    sorted.reserve(order.size());
    for (size_t i : order) sorted.push_back(std::move(seq[i]));
    std::move(sorted.begin(), sorted.end(), seq.begin());
    }
}
",
    },
    Piece {
        name: "sort_cmp",
        needs: &["fail"],
        code: "\
// `sort(cmp: lt)`: stable O(n log n) comparator sort. Under `LOCAL` (`--checked`),
// traps on non-strict-weak-order comparators (irreflexivity, asymmetry, intransitivity).
template <class S, class F> void sort_cmp(S& seq, F&& cmp, bool desc) {
    auto lt = [&](const auto& a, const auto& b) -> bool {
        bool r = desc ? cmp(b, a) : cmp(a, b);
#ifdef LOCAL
        if (r && (desc ? cmp(a, b) : cmp(b, a))) fail(\"sort `cmp:` comparator is not a strict weak order: both cmp(a, b) and cmp(b, a) are true\");
#endif
        return r;
    };
#ifdef LOCAL
    for (const auto& x : seq) {
        if (cmp(x, x)) fail(\"sort `cmp:` comparator is not a strict weak order: cmp(x, x) is true\");
    }
#endif
    std::stable_sort(seq.begin(), seq.end(), lt);
#ifdef LOCAL
    for (size_t i = 1; i < seq.size(); i++) {
        if (lt(seq[i], seq[i - 1]) || lt(seq[i], seq[0]) || lt(seq.back(), seq[i - 1])) {
            fail(\"sort `cmp:` comparator is not a strict weak order: result is not ordered\");
        }
    }
#endif
}
",
    },
    Piece {
        name: "pop_val",
        needs: &["fail"],
        code: "\
template <class T> T pop_val(std::vector<T>& v) { if (v.empty()) fail(\"pop from an empty list\"); T x = std::move(v.back()); v.pop_back(); return x; }
",
    },
    Piece {
        name: "first_val",
        needs: &["fail"],
        code: "\
template <class T> T first_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
inline u8 first_val(const std::string& s) { if (s.empty()) fail(\"the string is empty\"); return (u8)s.front(); }
",
    },
    Piece {
        name: "last_val",
        needs: &["fail"],
        code: "\
template <class T> T last_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
inline u8 last_val(const std::string& s) { if (s.empty()) fail(\"the string is empty\"); return (u8)s.back(); }
",
    },
    Piece {
        name: "list_view",
        needs: &["fail", "sorted"],
        code: "\
// O(1) read-only slice/reverse view into a `List<T>` (pointer + length + reverse bit).
template <class T> struct ListView {
    using value_type = T;
    const T* ptr = nullptr;
    size_t n = 0;
    bool rev = false;
    ll k0 = 0;
    ListView() = default;
    ListView(const std::vector<T>& v, bool r = false) noexcept : ptr(v.data()), n(v.size()), rev(r) {}
    ListView(const T* p, size_t sz, bool r = false, ll k = 0) noexcept : ptr(p), n(sz), rev(r), k0(k) {}
    void cpl_list_view_tag() const {}
    size_t size() const noexcept { return n; }
    bool empty() const noexcept { return n == 0; }
    const T& operator[](size_t i) const noexcept { return rev ? ptr[n - 1 - i] : ptr[i]; }
    const T& front() const noexcept { return (*this)[0]; }
    const T& back() const noexcept { return (*this)[n - 1]; }
    ListView reversed() const noexcept { return ListView(ptr, n, !rev, k0); }
    struct Iter {
        const T* d; size_t i, n; bool rev;
        const T& operator*() const noexcept { return rev ? d[n - 1 - i] : d[i]; }
        Iter& operator++() noexcept { ++i; return *this; }
        bool operator!=(const Iter& o) const noexcept { return i != o.i; }
        bool operator==(const Iter& o) const noexcept { return i == o.i; }
    };
    Iter begin() const noexcept { return {ptr, 0, n, rev}; }
    Iter end() const noexcept { return {ptr, n, n, rev}; }
    Iter rbegin() const noexcept { return reversed().begin(); }
    Iter rend() const noexcept { return reversed().end(); }
    int cmp(ListView o) const {
        size_t m = n < o.n ? n : o.n;
        for (size_t i = 0; i < m; i++) { const T& a = (*this)[i], &b = o[i]; if (a < b) return -1; if (b < a) return 1; }
        return n < o.n ? -1 : (n > o.n ? 1 : 0);
    }
    friend bool operator==(ListView a, ListView b) { return a.size() == b.size() && a.cmp(b) == 0; }
    friend bool operator!=(ListView a, ListView b) { return !(a == b); }
    friend bool operator<(ListView a, ListView b) { return a.cmp(b) < 0; }
    friend bool operator<=(ListView a, ListView b) { return a.cmp(b) <= 0; }
    friend bool operator>(ListView a, ListView b) { return a.cmp(b) > 0; }
    friend bool operator>=(ListView a, ListView b) { return a.cmp(b) >= 0; }
};
template <class T> ListView<T> list_slice(ListView<T> v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.n) fail(\"slice outside the list\"); size_t off = v.rev ? v.n - (size_t)hi : (size_t)lo; return ListView<T>(v.ptr + off, (size_t)(hi - lo), v.rev, v.k0 + (ll)off); }
template <class T> ll key_lo(ListView<T> v) noexcept { return v.k0; }
template <class T> ll key_pos(ListView<T> v, ll k) { ll i = k - v.k0, n = (ll)v.n; if (i < 0 || i >= n) key_fail(k, v.k0, n); return v.rev ? n - 1 - i : i; }
template <class T> ll pos_key(ListView<T> v, ll p) noexcept { return v.k0 + (v.rev ? (ll)v.n - 1 - p : p); }
template <class T> ListView<T> key_slice(ListView<T> v, ll lo, ll hi) { if (lo < v.k0 || hi < lo || hi > v.k0 + (ll)v.n) fail(\"slice outside the list's keys\"); return ListView<T>(v.ptr + (lo - v.k0), (size_t)(hi - lo), v.rev, lo); }
template <class T> ListView<T> list_slice(const std::vector<T>& v, ll lo, ll hi) { return list_slice(ListView<T>(v), lo, hi); }
template <class T> ListView<T> key_slice(const std::vector<T>& v, ll lo, ll hi) { return key_slice(ListView<T>(v), lo, hi); }
template <class T> ListView<T> list_rev(const std::vector<T>& v) noexcept { return ListView<T>(v, true); }
template <class T> ListView<T> list_rev(ListView<T> v) noexcept { return v.reversed(); }
template <class T> std::vector<T> to_list(const std::vector<T>& v) { return v; }
template <class T> std::vector<T> to_list(ListView<T> v) { std::vector<T> o; o.reserve(v.n); append_list_view(o, v); return o; }
template <class T> void append_list_view(std::vector<T>& o, ListView<T> v) { if (!v.rev) o.insert(o.end(), v.ptr, v.ptr + v.n); else for (size_t i = v.n; i-- > 0;) o.push_back(v.ptr[i]); }
template <class T> std::vector<T> list_concat(ListView<T> a, ListView<T> b) { std::vector<T> o; o.reserve(a.size() + b.size()); append_list_view(o, a); append_list_view(o, b); return o; }
template <class T> std::vector<T> list_concat(ListView<T> a, const std::vector<T>& b) { return list_concat(a, ListView<T>(b)); }
template <class T> std::vector<T> list_concat(const std::vector<T>& a, ListView<T> b) { return list_concat(ListView<T>(a), b); }
template <class T> std::vector<T> list_concat(const std::vector<T>& a, const std::vector<T>& b) { return list_concat(ListView<T>(a), ListView<T>(b)); }
template <class T> const T& at(ListView<T> v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> std::optional<T> first(ListView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
template <class T> std::optional<T> last(ListView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> T first_val(ListView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
template <class T> T last_val(ListView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <class T> std::vector<T> sorted(ListView<T> v) { auto o = to_list(v); sort_seq(o.begin(), o.end()); return o; }
template <class T, class U> bool has(ListView<T> v, const U& x) { for (size_t i = 0; i < v.n; i++) if (v.ptr[i] == x) return true; return false; }
inline std::string list_to_str(ListView<u8> v) { std::string s; s.reserve(v.size()); for (u8 c : v) s.push_back((char)c); return s; }
",
    },
    Piece {
        name: "rekey",
        needs: &["list_view"],
        code: "\
template <class T> ListView<T> rekey(ListView<T> v, ll lo, ll hi) { if (hi - lo != (ll)v.n) fail(\"rekey needs as many keys as cells\"); return ListView<T>(v.ptr, v.n, v.rev, lo); }
template <class T> ListView<T> rekey(const std::vector<T>& v, ll lo, ll hi) { return rekey(ListView<T>(v), lo, hi); }
template <class T> struct Keyed : ListView<T> {
    std::vector<T> store;
    Keyed() = default;
    Keyed(std::vector<T> v, ll k) : store(std::move(v)) { this->ptr = store.data(); this->n = store.size(); this->k0 = k; }
    Keyed(const Keyed& o) : ListView<T>(o), store(o.store) { this->ptr = store.data(); }
    Keyed(Keyed&& o) noexcept : ListView<T>(o), store(std::move(o.store)) { this->ptr = store.data(); }
    Keyed& operator=(const Keyed& o) { ListView<T>::operator=(o); store = o.store; this->ptr = store.data(); return *this; }
    Keyed& operator=(Keyed&& o) noexcept { ListView<T>::operator=(o); store = std::move(o.store); this->ptr = store.data(); return *this; }
};
template <class T> Keyed<T> rekey_own(std::vector<T> v, ll lo, ll hi) { if (hi - lo != (ll)v.size()) fail(\"rekey needs as many keys as cells\"); return Keyed<T>(std::move(v), lo); }
",
    },
    Piece {
        name: "mut_view",
        needs: &["fail", "sorted"],
        code: "\
// O(1) fixed-length mutable slice view into a `List<T>` or 1D `[D]T` (pointer + length).
template <class T> struct MutView {
    using value_type = T;
    T* ptr = nullptr;
    size_t n = 0;
    ll k0 = 0;
    MutView() = default;
    MutView(T* p, size_t sz, ll k = 0) noexcept : ptr(p), n(sz), k0(k) {}
    void cpl_list_view_tag() const {}
    size_t size() const noexcept { return n; }
    bool empty() const noexcept { return n == 0; }
    T* data() noexcept { return ptr; }
    const T* data() const noexcept { return ptr; }
    T* begin() noexcept { return ptr; }
    T* end() noexcept { return ptr + n; }
    const T* begin() const noexcept { return ptr; }
    const T* end() const noexcept { return ptr + n; }
    T& operator[](size_t i) noexcept { return ptr[i]; }
    const T& operator[](size_t i) const noexcept { return ptr[i]; }
    T& front() noexcept { return ptr[0]; }
    const T& front() const noexcept { return ptr[0]; }
    T& back() noexcept { return ptr[n - 1]; }
    const T& back() const noexcept { return ptr[n - 1]; }
};
template <class T> MutView<T> mut_slice(std::vector<T>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.size()) fail(\"slice outside the list\"); return MutView<T>(v.data() + (size_t)lo, (size_t)(hi - lo), lo); }
template <class T, size_t N> MutView<T> mut_slice(std::array<T, N>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > N) fail(\"slice outside the list\"); return MutView<T>(v.data() + (size_t)lo, (size_t)(hi - lo), lo); }
template <class T> MutView<T> mut_slice(MutView<T>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.n) fail(\"slice outside the list\"); return MutView<T>(v.ptr + (size_t)lo, (size_t)(hi - lo), v.k0 + lo); }
template <class T> ll key_lo(const MutView<T>& v) noexcept { return v.k0; }
template <class T> ll key_pos(const MutView<T>& v, ll k) { ll i = k - v.k0, n = (ll)v.n; if (i < 0 || i >= n) key_fail(k, v.k0, n); return i; }
template <class T> ll pos_key(const MutView<T>& v, ll p) noexcept { return v.k0 + p; }
template <class T> ListView<T> key_slice(const MutView<T>& v, ll lo, ll hi) { if (lo < v.k0 || hi < lo || hi > v.k0 + (ll)v.n) fail(\"slice outside the list's keys\"); return ListView<T>(v.ptr + (lo - v.k0), (size_t)(hi - lo), false, lo); }
template <class T> T& at(MutView<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> const T& at(const MutView<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> std::vector<T> to_list(MutView<T> v) { return std::vector<T>(v.ptr, v.ptr + v.n); }
template <class T> std::optional<T> first(MutView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
template <class T> std::optional<T> last(MutView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> T first_val(MutView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
template <class T> T last_val(MutView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <class T> std::vector<T> sorted(MutView<T> v) { auto o = to_list(v); sort_seq(o.begin(), o.end()); return o; }
template <class T, class U> bool has(MutView<T> v, const U& x) { for (size_t i = 0; i < v.n; i++) if (v.ptr[i] == x) return true; return false; }
",
    },
    Piece {
        name: "list_concat_at",
        needs: &["at"],
        code: "\
template <class A, class B> auto list_concat_at(const A& a, const B& b, ll i) -> decltype(at(a, i)) { if (i < 0) fail(\"index outside the list\"); if ((size_t)i < a.size()) return a[(size_t)i]; return at(b, i - (ll)a.size()); }
",
    },
];
