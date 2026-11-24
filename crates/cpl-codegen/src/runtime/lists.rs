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
        name: "list_insert",
        needs: &[],
        code: "\
// `xs.insert(i, x)`, `i` already held to `0..=len`: one move of the tail up.
template <class S, class T> inline void list_insert(S& s, ll i, T&& x) {
    s.insert(s.begin() + i, std::forward<T>(x));
}
",
    },
    Piece {
        name: "list_remove",
        needs: &[],
        code: "\
// `xs.remove(i)`, `i` already held to `0..len`: the element, then one move of the tail down.
template <class S> inline typename S::value_type list_remove(S& s, ll i) {
    typename S::value_type x = std::move(s[i]);
    s.erase(s.begin() + i);
    return x;
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
    // size_t, not ll: a store to an `ll` cell can't alias the sizes, so a
    // loop over cells keeps them in registers and can vectorise.
    size_t n[K] = {};
    template <class... I> size_t at(I... i) const { size_t p = 0; int k = 0; ((p = p * n[k++] + (size_t)(ll)i), ...); return p; }
    template <class... I> T& operator()(I... i) { return d[at(i...)]; }
    template <class... I> const T& operator()(I... i) const { return d[at(i...)]; }
    // Indexed by an `Int` that nothing has proved in range.
    template <class... I> size_t in(I... i) const { ll p = 0; bool bad = 0; int k = 0; ((bad |= (unsigned long long)(ll)i >= (unsigned long long)n[k], p = p * (ll)n[k] + (ll)i, k++), ...); if (bad) fail(\"index outside the array\"); return p; }
    template <class... I> T& ck(I... i) { return d[in(i...)]; }
    template <class... I> const T& ck(I... i) const { return d[in(i...)]; }
    bool operator==(const Grid& o) const { return d == o.d; }
    bool operator<(const Grid& o) const { return d < o.d; }
};
template <class T, int K> Grid<T, K> grid(std::array<ll, K> n, const T& fill) {
    Grid<T, K> g; ll cells = 1;
    for (int k = 0; k < K; k++) { g.n[k] = (size_t)n[k]; cells *= n[k]; }
    g.d.assign((size_t)cells, fill);
    return g;
}
",
    },
    Piece {
        name: "zeros",
        needs: &[],
        code: "\
}
// new is malloc, as in libstdc++, so a vector can own calloc's cells, whose untouched pages stay unmapped.
void* operator new(size_t n) { if (void* p = malloc(n ? n : 1)) return p; throw std::bad_alloc(); }
void operator delete(void* p) noexcept { free(p); }
void operator delete(void* p, size_t) noexcept { free(p); }
namespace cpl {
template <class T> std::vector<T> zeros(size_t n) {
#ifdef __GLIBCXX__
    struct Z : std::vector<T> { Z(size_t n) { auto& m = this->_M_impl; if (n && !(m._M_start = (T*)calloc(n, sizeof(T)))) throw std::bad_alloc(); m._M_finish = m._M_end_of_storage = m._M_start + n; } };
    return Z(n);
#endif
    return std::vector<T>(n);
}
",
    },
    Piece {
        name: "zero_grid",
        needs: &["grid", "zeros"],
        code: "\
template <class T, int K> Grid<T, K> zero_grid(std::array<ll, K> n) { Grid<T, K> g; ll c = 1; for (int k = 0; k < K; k++) c *= g.n[k] = n[k]; g.d = zeros<T>(c); return g; }
",
    },
    Piece {
        name: "blank",
        needs: &["zeros"],
        code: "\
// Number cells a table comprehension writes before anything reads them.
template <class T> std::vector<T> blank(size_t n) {
#ifdef __GLIBCXX__
    struct B : std::vector<T> { B(size_t n) { auto& m = this->_M_impl; if (n && !(m._M_start = (T*)malloc(n * sizeof(T)))) throw std::bad_alloc(); m._M_finish = m._M_end_of_storage = m._M_start + n; } };
    return B(n);
#endif
    return std::vector<T>(n);
}
",
    },
    Piece {
        name: "blank_grid",
        needs: &["grid", "blank"],
        code: "\
template <class T, int K> Grid<T, K> blank_grid(std::array<ll, K> n) { Grid<T, K> g; ll c = 1; for (int k = 0; k < K; k++) c *= g.n[k] = n[k]; g.d = blank<T>(c); return g; }
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
template <class T, size_t K, class U> bool has(const std::array<T, K>& v, const U& x) { return std::find(v.begin(), v.end(), x) != v.end(); }
",
    },
    Piece {
        name: "sorted",
        needs: &[],
        code: "\
// The interpreter's merge sort, so `cpl profile` counts the same comparisons natively:
// runs of 7 by insertion, then the right half and the left, merged through `buf`, which
// holds the left half (n/2 cells, as in libstdc++'s stable_sort).
template <class It, class B, class L> void merge_sort(It b, size_t lo, size_t hi, B buf, L& lt) { if (hi - lo <= 7) { for (size_t i = lo + 1; i < hi; i++) { auto x = std::move(b[i]); size_t j = i; for (; j > lo && lt(x, b[j - 1]); j--) b[j] = std::move(b[j - 1]); b[j] = std::move(x); } return; } size_t m = lo + (hi - lo) / 2; merge_sort(b, m, hi, buf, lt); merge_sort(b, lo, m, buf, lt); auto i = buf, im = std::move(b + lo, b + m, buf); auto j = b + m, jm = b + hi, k = b + lo; for (; i != im && j != jm; ++k) { if (lt(*j, *i)) { *k = std::move(*j); ++j; } else { *k = std::move(*i); ++i; } } std::move(i, im, k); }
template <class It, class L> void stable_sort_lt(It b, It e, L lt) { using V = typename std::iterator_traits<It>::value_type; size_t n = e - b; std::vector<V> buf(b, b + n / 2); merge_sort(b, 0, n, buf.begin(), lt); }
// `Same`: elements that compare equal are the same value, so no order of them is told apart.
template <bool Same = false, class It, class L = std::less<>> void sort_seq(It b, It e, L lt = L()) { using V = typename std::iterator_traits<It>::value_type; if constexpr (Same || std::is_integral_v<V>) std::sort(b, e, lt); else stable_sort_lt(b, e, lt); }
// Elements already in strictly ascending order are told apart by it, and it is the order
// they had, so `std::sort` with ties broken by it gives the stable sort's output.
template <class It, class L> void sort_distinct(It b, It e, L lt) { if (std::adjacent_find(b, e, [](const auto& x, const auto& y) { return !(x < y); }) != e) return stable_sort_lt(b, e, lt); std::sort(b, e, [&](const auto& x, const auto& y) { return lt(x, y) || (!lt(y, x) && x < y); }); }
template <bool Same = false, class T> std::vector<T> sorted(std::vector<T> v) { sort_seq<Same>(v.begin(), v.end()); return v; }
template <bool Same = false, class T> std::vector<T> sorted(const std::set<T>& s) { return std::vector<T>(s.begin(), s.end()); }
template <bool Same = false, class T, size_t K> std::vector<T> sorted(const std::array<T, K>& a) { std::vector<T> v(a.begin(), a.end()); sort_seq<Same>(v.begin(), v.end()); return v; }
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
        name: "list_to_fixed",
        needs: &["fail"],
        code: "\
// A list of exactly K cells as the `std::array` its fixed-size array type is held in;
// cells move out of a list nothing else reads.
template <size_t K, class V> auto list_to_fixed(V&& v) {
    using T = typename std::decay_t<V>::value_type;
    if (v.size() != K) fail(\"a list of another length than its array has\");
    std::array<T, K> a;
    if constexpr (std::is_rvalue_reference_v<V&&>) std::move(v.begin(), v.end(), a.begin());
    else std::copy(v.begin(), v.end(), a.begin());
    return a;
}
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
// `Ties`: ties are broken by position, so `std::sort` gives the stable order.
template <bool Ties = false, class S, class F> void sort_by(S& seq, F&& key, bool desc) {
    using K = std::decay_t<decltype(key(*seq.begin()))>;
    using T = typename S::value_type;
    if constexpr (std::is_void_v<K>) {
        for (const auto& x : seq) key(x);
        (void)desc;
        return;
    } else {
    // Each key sits next to its cell's index, so a comparison reads one
    // contiguous pair. No list in memory holds 2^32 cells.
    std::vector<std::pair<K, uint32_t>> keyed;
    keyed.reserve(seq.size());
    for (const auto& x : seq) keyed.emplace_back(key(x), (uint32_t)keyed.size());
    if constexpr (Ties) {
        if (desc) std::sort(keyed.begin(), keyed.end(), [](const auto& a, const auto& b) { return b.first < a.first || (!(a.first < b.first) && a.second < b.second); });
        else std::sort(keyed.begin(), keyed.end());
    } else if (desc) stable_sort_lt(keyed.begin(), keyed.end(), [](const auto& a, const auto& b) { return b.first < a.first; });
    else stable_sort_lt(keyed.begin(), keyed.end(), [](const auto& a, const auto& b) { return a.first < b.first; });
    std::vector<uint32_t> order(keyed.size());
    for (size_t i = 0; i < order.size(); i++) order[i] = keyed[i].second;
    // seq[j] takes the old seq[order[j]], one cycle of the permutation at a time.
    for (size_t i = 0; i < order.size(); i++) {
        if (order[i] == i) continue;
        T x = std::move(seq[i]);
        size_t j = i;
        for (size_t k; (k = order[j]) != i; j = k) { seq[j] = std::move(seq[k]); order[j] = j; }
        seq[j] = std::move(x);
        order[j] = j;
    }
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
template <class T, size_t K> T first_val(const std::array<T, K>& v) { return v.front(); }
inline u8 first_val(const std::string& s) { if (s.empty()) fail(\"the string is empty\"); return (u8)s.front(); }
",
    },
    Piece {
        name: "last_val",
        needs: &["fail"],
        code: "\
template <class T> T last_val(const std::vector<T>& v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <class T, size_t K> T last_val(const std::array<T, K>& v) { return v.back(); }
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
    size_t st = 1;
    ListView() = default;
    ListView(const std::vector<T>& v, bool r = false) noexcept : ptr(v.data()), n(v.size()), rev(r) {}
    template <size_t K> ListView(const std::array<T, K>& v, bool r = false) noexcept : ptr(v.data()), n(K), rev(r) {}
    ListView(const T* p, size_t sz, bool r = false, ll k = 0, size_t s = 1) noexcept : ptr(p), n(sz), rev(r), k0(k), st(s) {}
    void cpl_list_view_tag() const {}
    size_t size() const noexcept { return n; }
    bool empty() const noexcept { return n == 0; }
    const T& operator[](size_t i) const noexcept { return ptr[(rev ? n - 1 - i : i) * st]; }
    const T& front() const noexcept { return (*this)[0]; }
    const T& back() const noexcept { return (*this)[n - 1]; }
    ListView reversed() const noexcept { return ListView(ptr, n, !rev, k0, st); }
    struct Iter {
        const T* d; size_t i, n; bool rev; size_t st;
        const T& operator*() const noexcept { return d[(rev ? n - 1 - i : i) * st]; }
        Iter& operator++() noexcept { ++i; return *this; }
        bool operator!=(const Iter& o) const noexcept { return i != o.i; }
        bool operator==(const Iter& o) const noexcept { return i == o.i; }
    };
    Iter begin() const noexcept { return {ptr, 0, n, rev, st}; }
    Iter end() const noexcept { return {ptr, n, n, rev, st}; }
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
template <class T> ListView<T> list_slice(ListView<T> v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.n) fail(\"slice outside the list\"); size_t off = v.rev ? v.n - (size_t)hi : (size_t)lo; return ListView<T>(v.ptr + off * v.st, (size_t)(hi - lo), v.rev, v.k0 + (ll)off, v.st); }
template <class T> ll key_lo(ListView<T> v) noexcept { return v.k0; }
template <class T> ll key_pos(ListView<T> v, ll k) { ll i = k - v.k0, n = (ll)v.n; if (i < 0 || i >= n) key_fail(k, v.k0, n); return v.rev ? n - 1 - i : i; }
template <class T> ll pos_key(ListView<T> v, ll p) noexcept { return v.k0 + (v.rev ? (ll)v.n - 1 - p : p); }
template <class T> ListView<T> key_slice(ListView<T> v, ll lo, ll hi) { if (lo < v.k0 || hi < lo || hi > v.k0 + (ll)v.n) fail(\"slice outside the list's keys\"); return ListView<T>(v.ptr + (size_t)(lo - v.k0) * v.st, (size_t)(hi - lo), v.rev, lo, v.st); }
template <class T> ListView<T> list_slice(const std::vector<T>& v, ll lo, ll hi) { return list_slice(ListView<T>(v), lo, hi); }
template <class T, size_t K> ListView<T> list_slice(const std::array<T, K>& v, ll lo, ll hi) { return list_slice(ListView<T>(v), lo, hi); }
template <class T, size_t K> ListView<T> key_slice(const std::array<T, K>& v, ll lo, ll hi) { return key_slice(ListView<T>(v), lo, hi); }
template <class T, size_t K> ListView<T> list_rev(const std::array<T, K>& v) noexcept { return ListView<T>(v, true); }
template <class T, size_t K> std::vector<T> to_list(const std::array<T, K>& v) { return std::vector<T>(v.begin(), v.end()); }
template <class T> ListView<T> key_slice(const std::vector<T>& v, ll lo, ll hi) { return key_slice(ListView<T>(v), lo, hi); }
template <class T> ListView<T> list_rev(const std::vector<T>& v) noexcept { return ListView<T>(v, true); }
template <class T> ListView<T> list_rev(ListView<T> v) noexcept { return v.reversed(); }
template <class T> std::vector<T> to_list(const std::vector<T>& v) { return v; }
template <class T> std::vector<T> to_list(ListView<T> v) { std::vector<T> o; o.reserve(v.n); append_list_view(o, v); return o; }
template <class T> void append_list_view(std::vector<T>& o, ListView<T> v) { if (!v.rev && v.st == 1) o.insert(o.end(), v.ptr, v.ptr + v.n); else for (size_t i = 0; i < v.n; i++) o.push_back(v[i]); }
template <class T> std::vector<T> list_concat(ListView<T> a, ListView<T> b) { std::vector<T> o; o.reserve(a.size() + b.size()); append_list_view(o, a); append_list_view(o, b); return o; }
template <class T> std::vector<T> list_concat(ListView<T> a, const std::vector<T>& b) { return list_concat(a, ListView<T>(b)); }
template <class T> std::vector<T> list_concat(const std::vector<T>& a, ListView<T> b) { return list_concat(ListView<T>(a), b); }
template <class T> std::vector<T> list_concat(const std::vector<T>& a, const std::vector<T>& b) { return list_concat(ListView<T>(a), ListView<T>(b)); }
template <class T> const T& at(ListView<T> v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> std::optional<T> first(ListView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
template <class T> std::optional<T> last(ListView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> T first_val(ListView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
template <class T> T last_val(ListView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <bool Same = false, class T> std::vector<T> sorted(ListView<T> v) { auto o = to_list(v); sort_seq<Same>(o.begin(), o.end()); return o; }
template <class T, class U> bool has(ListView<T> v, const U& x) { for (size_t i = 0; i < v.n; i++) if (v.ptr[i * v.st] == x) return true; return false; }
inline std::string list_to_str(ListView<u8> v) { std::string s; s.reserve(v.size()); for (u8 c : v) s.push_back((char)c); return s; }
",
    },
    Piece {
        name: "rekey",
        needs: &["list_view"],
        code: "\
template <class T> ListView<T> rekey(ListView<T> v, ll lo, ll hi) { if (hi - lo != (ll)v.n) fail(\"rekey needs as many keys as cells\"); return ListView<T>(v.ptr, v.n, v.rev, lo, v.st); }
template <class T> ListView<T> rekey(const std::vector<T>& v, ll lo, ll hi) { return rekey(ListView<T>(v), lo, hi); }
template <class T, size_t K> ListView<T> rekey(const std::array<T, K>& v, ll lo, ll hi) { return rekey(ListView<T>(v), lo, hi); }
template <class T> struct Keyed : ListView<T> {
    std::vector<T> store;
    // False when the cells belong to another value, as a grid's row or column does.
    bool own = true;
    Keyed() = default;
    Keyed(std::vector<T> v, ll k) : store(std::move(v)) { this->ptr = store.data(); this->n = store.size(); this->k0 = k; }
    explicit Keyed(ListView<T> v) noexcept : ListView<T>(v), own(false) {}
    Keyed(const Keyed& o) : ListView<T>(o), store(o.store), own(o.own) { if (own) this->ptr = store.data(); }
    Keyed(Keyed&& o) noexcept : ListView<T>(o), store(std::move(o.store)), own(o.own) { if (own) this->ptr = store.data(); }
    Keyed& operator=(const Keyed& o) { ListView<T>::operator=(o); store = o.store; own = o.own; if (own) this->ptr = store.data(); return *this; }
    Keyed& operator=(Keyed&& o) noexcept { ListView<T>::operator=(o); store = std::move(o.store); own = o.own; if (own) this->ptr = store.data(); return *this; }
};
template <class T> Keyed<T> rekey_own(std::vector<T> v, ll lo, ll hi) { if (hi - lo != (ll)v.size()) fail(\"rekey needs as many keys as cells\"); return Keyed<T>(std::move(v), lo); }
",
    },
    Piece {
        name: "mut_view",
        needs: &["fail", "sorted"],
        code: "\
// A random-access iterator over every `s`-th cell from `p`: a grid column when `s` > 1.
template <class P> struct StepIt {
    using iterator_category = std::random_access_iterator_tag;
    using value_type = std::remove_cv_t<std::remove_pointer_t<P>>;
    using difference_type = std::ptrdiff_t;
    using pointer = P;
    using reference = std::remove_pointer_t<P>&;
    P p; difference_type i, s;
    reference operator*() const noexcept { return p[i * s]; }
    pointer operator->() const noexcept { return p + i * s; }
    reference operator[](difference_type k) const noexcept { return p[(i + k) * s]; }
    StepIt& operator++() noexcept { ++i; return *this; }
    StepIt operator++(int) noexcept { StepIt t = *this; ++i; return t; }
    StepIt& operator--() noexcept { --i; return *this; }
    StepIt operator--(int) noexcept { StepIt t = *this; --i; return t; }
    StepIt& operator+=(difference_type k) noexcept { i += k; return *this; }
    StepIt& operator-=(difference_type k) noexcept { i -= k; return *this; }
    friend StepIt operator+(StepIt a, difference_type k) noexcept { a.i += k; return a; }
    friend StepIt operator+(difference_type k, StepIt a) noexcept { a.i += k; return a; }
    friend StepIt operator-(StepIt a, difference_type k) noexcept { a.i -= k; return a; }
    friend difference_type operator-(const StepIt& a, const StepIt& b) noexcept { return a.i - b.i; }
    friend bool operator==(const StepIt& a, const StepIt& b) noexcept { return a.i == b.i; }
    friend bool operator!=(const StepIt& a, const StepIt& b) noexcept { return a.i != b.i; }
    friend bool operator<(const StepIt& a, const StepIt& b) noexcept { return a.i < b.i; }
    friend bool operator>(const StepIt& a, const StepIt& b) noexcept { return a.i > b.i; }
    friend bool operator<=(const StepIt& a, const StepIt& b) noexcept { return a.i <= b.i; }
    friend bool operator>=(const StepIt& a, const StepIt& b) noexcept { return a.i >= b.i; }
};
// O(1) fixed-length mutable view into a `List<T>`, 1D `[D]T` or a grid's row or column
// (pointer + length + stride).
template <class T> struct MutView {
    using value_type = T;
    T* ptr = nullptr;
    size_t n = 0;
    ll k0 = 0;
    size_t st = 1;
    MutView() = default;
    MutView(T* p, size_t sz, ll k = 0, size_t s = 1) noexcept : ptr(p), n(sz), k0(k), st(s) {}
    void cpl_list_view_tag() const {}
    size_t size() const noexcept { return n; }
    bool empty() const noexcept { return n == 0; }
    T* data() noexcept { return ptr; }
    const T* data() const noexcept { return ptr; }
    StepIt<T*> begin() noexcept { return {ptr, 0, (std::ptrdiff_t)st}; }
    StepIt<T*> end() noexcept { return {ptr, (std::ptrdiff_t)n, (std::ptrdiff_t)st}; }
    StepIt<const T*> begin() const noexcept { return {ptr, 0, (std::ptrdiff_t)st}; }
    StepIt<const T*> end() const noexcept { return {ptr, (std::ptrdiff_t)n, (std::ptrdiff_t)st}; }
    T& operator[](size_t i) noexcept { return ptr[i * st]; }
    const T& operator[](size_t i) const noexcept { return ptr[i * st]; }
    T& front() noexcept { return ptr[0]; }
    const T& front() const noexcept { return ptr[0]; }
    T& back() noexcept { return ptr[(n - 1) * st]; }
    const T& back() const noexcept { return ptr[(n - 1) * st]; }
};
template <class T> MutView<T> mut_slice(std::vector<T>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.size()) fail(\"slice outside the list\"); return MutView<T>(v.data() + (size_t)lo, (size_t)(hi - lo), lo); }
template <class T, size_t N> MutView<T> mut_slice(std::array<T, N>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > N) fail(\"slice outside the list\"); return MutView<T>(v.data() + (size_t)lo, (size_t)(hi - lo), lo); }
template <class T> MutView<T> mut_slice(MutView<T>& v, ll lo, ll hi) { if (lo < 0 || hi < lo || (size_t)hi > v.n) fail(\"slice outside the list\"); return MutView<T>(v.ptr + (size_t)lo * v.st, (size_t)(hi - lo), v.k0 + lo, v.st); }
template <class T> ll key_lo(const MutView<T>& v) noexcept { return v.k0; }
template <class T> ll key_pos(const MutView<T>& v, ll k) { ll i = k - v.k0, n = (ll)v.n; if (i < 0 || i >= n) key_fail(k, v.k0, n); return i; }
template <class T> ll pos_key(const MutView<T>& v, ll p) noexcept { return v.k0 + p; }
template <class T> ListView<T> key_slice(const MutView<T>& v, ll lo, ll hi) { if (lo < v.k0 || hi < lo || hi > v.k0 + (ll)v.n) fail(\"slice outside the list's keys\"); return ListView<T>(v.ptr + (size_t)(lo - v.k0) * v.st, (size_t)(hi - lo), false, lo, v.st); }
template <class T> T& at(MutView<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> const T& at(const MutView<T>& v, ll i) { if ((unsigned long long)i >= v.size()) fail(\"index outside the list\"); return v[(size_t)i]; }
template <class T> std::vector<T> to_list(MutView<T> v) { return std::vector<T>(v.begin(), v.end()); }
template <class T> std::optional<T> first(MutView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.front()); }
template <class T> std::optional<T> last(MutView<T> v) { return v.empty() ? std::optional<T>() : std::optional<T>(v.back()); }
template <class T> T first_val(MutView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.front(); }
template <class T> T last_val(MutView<T> v) { if (v.empty()) fail(\"the list is empty\"); return v.back(); }
template <bool Same = false, class T> std::vector<T> sorted(MutView<T> v) { auto o = to_list(v); sort_seq<Same>(o.begin(), o.end()); return o; }
template <class T, class U> bool has(MutView<T> v, const U& x) { for (size_t i = 0; i < v.n; i++) if (v.ptr[i * v.st] == x) return true; return false; }
",
    },
    Piece {
        name: "grid_line",
        needs: &["grid", "rekey", "mut_view"],
        code: "\
// Row `at` (axis 1 whole) or column `at` (axis 0 whole) of a 2D grid: its cells, not a copy.
template <class T> ListView<T> grid_cells(const Grid<T, 2>& g, ll at, ll axis) { size_t fixed = g.n[axis == 1 ? 0 : 1]; if ((unsigned long long)at >= fixed) fail(\"index outside the array\"); return axis == 1 ? ListView<T>(g.d.data() + (size_t)at * g.n[1], g.n[1]) : ListView<T>(g.d.data() + (size_t)at, g.n[0], false, 0, g.n[1]); }
template <class T> Keyed<T> grid_line(const Grid<T, 2>& g, ll at, ll axis) { return Keyed<T>(grid_cells(g, at, axis)); }
template <class T> MutView<T> mut_line(Grid<T, 2>& g, ll at, ll axis) { ListView<T> v = grid_cells(g, at, axis); return MutView<T>(g.d.data() + (v.ptr - g.d.data()), v.n, 0, v.st); }
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
