//! Maps and sets: ordered lookups, the hash used for unordered ones, and
//! interval maps.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
    Piece {
        name: "get",
        needs: &[],
        code: "\
template <class K, class V, class Q> std::optional<V> get(const std::map<K, V>& m, const Q& k) { auto it = m.find(k); return it == m.end() ? std::optional<V>() : std::optional<V>(it->second); }
",
    },
    Piece {
        name: "set_min",
        needs: &[],
        code: "\
template <class T> std::optional<T> set_min(const std::set<T>& s) { return s.empty() ? std::optional<T>() : std::optional<T>(*s.begin()); }
",
    },
    Piece {
        name: "ordered_near",
        needs: &[],
        code: "\
// The nearest key of an ordered set, or entry of an ordered map, on one side of `k`.
template <class T> T near_entry(const std::set<T>&, const T& key) { return key; }
template <class K, class V> std::tuple<K, V> near_entry(const std::map<K, V>&, const std::pair<const K, V>& e) { return std::tuple<K, V>(e.first, e.second); }
template <class C> auto near_at(const C& c, typename C::const_iterator it) -> std::optional<decltype(near_entry(c, *it))> {
    if (it == c.end()) return std::nullopt;
    return near_entry(c, *it);
}
template <class C> auto near_before(const C& c, typename C::const_iterator it) -> std::optional<decltype(near_entry(c, *it))> {
    if (it == c.begin()) return std::nullopt;
    return near_entry(c, *std::prev(it));
}
template <class C, class Q> auto first_ge(const C& c, const Q& k) { return near_at(c, c.lower_bound(k)); }
template <class C, class Q> auto first_gt(const C& c, const Q& k) { return near_at(c, c.upper_bound(k)); }
template <class C, class Q> auto last_le(const C& c, const Q& k) { return near_before(c, c.upper_bound(k)); }
template <class C, class Q> auto last_lt(const C& c, const Q& k) { return near_before(c, c.lower_bound(k)); }
template <class C> auto first_of(const C& c) { return near_at(c, c.begin()); }
template <class C> auto last_of(const C& c) { return near_before(c, c.end()); }
template <class C> auto pop_first(C& c) { auto r = first_of(c); if (r) c.erase(c.begin()); return r; }
template <class C> auto pop_last(C& c) { auto r = last_of(c); if (r) c.erase(std::prev(c.end())); return r; }
",
    },
    Piece {
        name: "set_max",
        needs: &[],
        code: "\
template <class T> std::optional<T> set_max(const std::set<T>& s) { return s.empty() ? std::optional<T>() : std::optional<T>(*s.rbegin()); }
",
    },
    Piece {
        name: "cphash",
        needs: &[],
        code: "\
// Salted splitmix64 hash for HashMap / HashSet (defeats anti-hash tests).
struct CpHash {
    static uint64_t splitmix64(uint64_t x) {
        x += 0x9e3779b97f4a7c15ULL;
        x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
        x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
        return x ^ (x >> 31);
    }
    static uint64_t salt() {
        static const uint64_t s = (uint64_t)std::chrono::steady_clock::now().time_since_epoch().count();
        return s;
    }
    // Full salted splitmix64 so both prime-mod and power-of-two open-addressing tables stay hack-resistant.
    size_t operator()(ll x) const { return (size_t)splitmix64((uint64_t)x + salt()); }
    size_t operator()(int x) const { return (*this)((ll)x); }
    size_t operator()(u8 x) const { return (*this)((ll)x); }
    size_t operator()(bool x) const { return (*this)((ll)x); }
    size_t operator()(double x) const { return (*this)((ll)std::hash<double>()(x)); }
    size_t operator()(const std::string& s) const {
        uint64_t h = salt();
        for (unsigned char c : s) h = splitmix64(h ^ c);
        return (size_t)h;
    }
    // A string view: same hash as `std::string`.
    template <class S> auto operator()(const S& s) const -> decltype(s.cpl_str_view_tag(), size_t()) {
        uint64_t h = salt();
        for (u8 c : s) h = splitmix64(h ^ (unsigned char)c);
        return (size_t)h;
    }
    // A list, list view, or fixed array: its length, then each element in order.
    template <class L> auto operator()(const L& xs) const -> decltype(xs.begin(), xs.size(), xs.cpl_list_view_tag(), size_t()) {
        uint64_t h = salt() ^ xs.size();
        for (const auto& x : xs) h = splitmix64(h ^ (uint64_t)(*this)(x));
        return (size_t)h;
    }
    template <class L> auto operator()(const L& xs) const -> decltype(xs.begin(), xs.data(), size_t()) {
        uint64_t h = salt() ^ xs.size();
        for (const auto& x : xs) h = splitmix64(h ^ (uint64_t)(*this)(x));
        return (size_t)h;
    }
    // An optional, packed or not.
    template <class O> auto operator()(const O& o) const -> decltype(o.has_value(), size_t()) { return o ? (*this)(*o) : (size_t)splitmix64(salt()); }
    // A hash set: the same for any order of its elements, so equal sets hash equal.
    template <class S> auto operator()(const S& s) const -> decltype(s.count(*s.begin()), size_t()) { uint64_t h = salt() ^ s.size(); for (const auto& x : s) h += splitmix64((uint64_t)(*this)(x)); return (size_t)splitmix64(h); }
    template <class T, size_t I = 0> uint64_t hash_tuple(const T& t) const {
        if constexpr (I == std::tuple_size<T>::value) return salt();
        else return splitmix64(hash_tuple<T, I + 1>(t) ^ (uint64_t)(*this)(std::get<I>(t)));
    }
    template <class... Ts> size_t operator()(const std::tuple<Ts...>& t) const { return (size_t)hash_tuple(t); }
    template <class T> auto operator()(const T& x) const -> decltype((*this)(x.cpl_tie())) {
        return (*this)(x.cpl_tie());
    }
};
",
    },
    Piece {
        name: "cphashmap",
        needs: &["cphash", "fail"],
        code: "\
// Open-addressing hash map with 7-bit fingerprints and backward-shift deletion.
template <class K, class V, class H = CpHash> struct HashMap {
    using key_type = K; using mapped_type = V; using value_type = std::pair<K, V>;
    std::vector<uint8_t> st; std::vector<value_type> kv; size_t n = 0;
    template <bool C> struct It {
        using P = std::conditional_t<C, const value_type, value_type>;
        const uint8_t* s; P* p; size_t i, cap;
        void skip() { while (i < cap && !s[i]) ++i; }
        P& operator*() const { return p[i]; }
        P* operator->() const { return p + i; }
        It& operator++() { ++i; skip(); return *this; }
        bool operator==(const It& o) const { return i == o.i; }
        bool operator!=(const It& o) const { return i != o.i; }
    };
    using Iter = It<false>; using CIter = It<true>;
    size_t size() const { return n; }
    bool empty() const { return n == 0; }
    void clear() {
        if (!n) return;
        if (st.size() > 64 && n * 8 < st.size()) { st.clear(); kv.clear(); }
        else std::fill(st.begin(), st.end(), 0);
        n = 0;
    }
    void rehash(size_t cap) {
        std::vector<uint8_t> ns(cap, 0); std::vector<value_type> nk(cap); size_t m = cap - 1;
        for (size_t i = 0; i < st.size(); ++i) if (st[i]) {
            size_t j = H{}(kv[i].first) & m;
            while (ns[j]) j = (j + 1) & m;
            ns[j] = st[i]; nk[j] = std::move(kv[i]);
        }
        st.swap(ns); kv.swap(nk);
    }
    size_t locate(const K& k) const {
        if (st.empty()) return 0;
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && kv[i].first == k) return i; i = (i + 1) & m; }
        return st.size();
    }
    std::pair<size_t, bool> slot(const K& k) {
        if (2 * (n + 1) > st.size()) { K kc = k; rehash(st.empty() ? 8 : st.size() * 2); return slot(kc); }
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && kv[i].first == k) return {i, false}; i = (i + 1) & m; }
        st[i] = fp; kv[i].first = k; ++n; return {i, true};
    }
    Iter begin() { Iter it{st.data(), kv.data(), 0, st.size()}; it.skip(); return it; }
    Iter end() { return {st.data(), kv.data(), st.size(), st.size()}; }
    CIter begin() const { CIter it{st.data(), kv.data(), 0, st.size()}; it.skip(); return it; }
    CIter end() const { return {st.data(), kv.data(), st.size(), st.size()}; }
    Iter find(const K& k) { return {st.data(), kv.data(), locate(k), st.size()}; }
    CIter find(const K& k) const { return {st.data(), kv.data(), locate(k), st.size()}; }
    size_t count(const K& k) const { return locate(k) != st.size() ? 1 : 0; }
    V& at(const K& k) { size_t i = locate(k); if (i == st.size()) fail(\"key not in map\"); return kv[i].second; }
    const V& at(const K& k) const { size_t i = locate(k); if (i == st.size()) fail(\"key not in map\"); return kv[i].second; }
    V& operator[](const K& k) { auto [i, nw] = slot(k); if (nw) kv[i].second = V{}; return kv[i].second; }
    std::pair<Iter, bool> emplace(const K& k, const V& v) {
        if (2 * (n + 1) > st.size()) { K kc = k; V vc = v; rehash(st.empty() ? 8 : st.size() * 2); return emplace(kc, vc); }
        auto [i, nw] = slot(k); if (nw) kv[i].second = v;
        return {{st.data(), kv.data(), i, st.size()}, nw};
    }
    void erase_at(size_t i) {
        --n; size_t m = st.size() - 1, j = i;
        for (;;) {
            j = (j + 1) & m; if (!st[j]) break;
            size_t home = H{}(kv[j].first) & m;
            if (((j - home) & m) >= ((j - i) & m)) { st[i] = st[j]; kv[i] = std::move(kv[j]); i = j; }
        }
        st[i] = 0; kv[i] = {};
    }
    void erase(Iter it) { erase_at(it.i); }
    size_t erase(const K& k) {
        size_t i = locate(k); if (i == st.size()) return 0;
        erase_at(i); return 1;
    }
    bool operator==(const HashMap& o) const {
        if (n != o.n) return false;
        for (size_t i = 0; i < st.size(); ++i) if (st[i]) {
            size_t j = o.locate(kv[i].first);
            if (j == o.st.size() || !(o.kv[j].second == kv[i].second)) return false;
        }
        return true;
    }
    bool operator!=(const HashMap& o) const { return !(*this == o); }
};
",
    },
    Piece {
        name: "cphashset",
        needs: &["cphash"],
        code: "\
// Open-addressing hash set with 7-bit fingerprints and backward-shift deletion.
template <class K, class H = CpHash> struct HashSet {
    using key_type = K; using value_type = K;
    std::vector<uint8_t> st; std::vector<std::array<K, 1>> ks; size_t n = 0;
    struct CIter {
        const uint8_t* s; const std::array<K, 1>* p; size_t i, cap;
        void skip() { while (i < cap && !s[i]) ++i; }
        const K& operator*() const { return p[i][0]; }
        const K* operator->() const { return &p[i][0]; }
        CIter& operator++() { ++i; skip(); return *this; }
        bool operator==(const CIter& o) const { return i == o.i; }
        bool operator!=(const CIter& o) const { return i != o.i; }
    };
    size_t size() const { return n; }
    bool empty() const { return n == 0; }
    void clear() {
        if (!n) return;
        if (st.size() > 64 && n * 8 < st.size()) { st.clear(); ks.clear(); }
        else std::fill(st.begin(), st.end(), 0);
        n = 0;
    }
    void rehash(size_t cap) {
        std::vector<uint8_t> ns(cap, 0); std::vector<std::array<K, 1>> nk(cap); size_t m = cap - 1;
        for (size_t i = 0; i < st.size(); ++i) if (st[i]) {
            size_t j = H{}(ks[i][0]) & m;
            while (ns[j]) j = (j + 1) & m;
            ns[j] = st[i]; nk[j][0] = std::move(ks[i][0]);
        }
        st.swap(ns); ks.swap(nk);
    }
    size_t locate(const K& k) const {
        if (st.empty()) return 0;
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && ks[i][0] == k) return i; i = (i + 1) & m; }
        return st.size();
    }
    CIter begin() const { CIter it{st.data(), ks.data(), 0, st.size()}; it.skip(); return it; }
    CIter end() const { return {st.data(), ks.data(), st.size(), st.size()}; }
    CIter find(const K& k) const { return {st.data(), ks.data(), locate(k), st.size()}; }
    size_t count(const K& k) const { return locate(k) != st.size() ? 1 : 0; }
    std::pair<CIter, bool> insert(const K& k) {
        if (2 * (n + 1) > st.size()) { K kc = k; rehash(st.empty() ? 8 : st.size() * 2); return insert(kc); }
        size_t h = H{}(k), m = st.size() - 1, i = h & m; uint8_t fp = (uint8_t)(128 | (h >> 57));
        while (st[i]) { if (st[i] == fp && ks[i][0] == k) return {{st.data(), ks.data(), i, st.size()}, false}; i = (i + 1) & m; }
        st[i] = fp; ks[i][0] = k; ++n; return {{st.data(), ks.data(), i, st.size()}, true};
    }
    size_t erase(const K& k) {
        size_t i = locate(k); if (i == st.size()) return 0;
        --n; size_t m = st.size() - 1, j = i;
        for (;;) {
            j = (j + 1) & m; if (!st[j]) break;
            size_t home = H{}(ks[j][0]) & m;
            if (((j - home) & m) >= ((j - i) & m)) { st[i] = st[j]; ks[i][0] = std::move(ks[j][0]); i = j; }
        }
        st[i] = 0; ks[i][0] = K{}; return 1;
    }
    bool operator==(const HashSet& o) const {
        if (n != o.n) return false;
        for (size_t i = 0; i < st.size(); ++i) if (st[i] && !o.count(ks[i][0])) return false;
        return true;
    }
    bool operator!=(const HashSet& o) const { return !(*this == o); }
};
template <class K, class H> std::vector<K> sorted(const HashSet<K, H>& s) { std::vector<K> v; v.reserve(s.size()); for (const K& k : s) v.push_back(k); sort_seq(v.begin(), v.end()); return v; }
",
    },
    Piece {
        name: "map_get_any",
        needs: &[],
        code: "\
template <class M, class Q> auto map_get(const M& m, const Q& k) -> std::optional<typename M::mapped_type> {
    auto it = m.find(k); return it == m.end() ? std::nullopt : std::optional<typename M::mapped_type>(it->second);
}
",
    },
    Piece {
        name: "map_lend",
        needs: &[],
        code: "\
template <class V> struct Lent {
    const V* p;
    bool has_value() const { return p != nullptr; }
    explicit operator bool() const { return p != nullptr; }
    const V& operator*() const { return *p; }
    const V* operator->() const { return p; }
    operator std::optional<V>() const { return p ? std::optional<V>(*p) : std::nullopt; }
};
template <class M, class Q> Lent<typename M::mapped_type> map_lend(const M& m, const Q& k) {
    auto it = m.find(k); return {it == m.end() ? nullptr : &it->second};
}
",
    },
    Piece {
        name: "map_or",
        needs: &[],
        code: "\
template <class M, class Q> const typename M::mapped_type& map_or(const M& m, const Q& k, const typename M::mapped_type& d) {
    auto it = m.find(k); return it == m.end() ? d : it->second;
}
",
    },
    Piece {
        name: "map_move_or",
        needs: &[],
        code: "\
template <class M, class Q> typename M::mapped_type map_move_or(M& m, const Q& k, typename M::mapped_type d) {
    auto it = m.find(k); if (it == m.end()) return d; return std::move(it->second);
}
",
    },
    Piece {
        name: "map_take",
        needs: &[],
        code: "\
template <class M, class Q> auto map_take(M& m, const Q& k) -> std::optional<typename M::mapped_type> {
    auto it = m.find(k); if (it == m.end()) return std::nullopt;
    auto v = std::move(it->second); m.erase(it); return v;
}
",
    },
    Piece {
        name: "interval_assign",
        needs: &["fail", "range"],
        code: "\
template <class M, class Pos, class V, class F>
void interval_assign(M& segs, const Range<Pos>& r, const V& val, F&& on_old) {
    if (r.lo > r.hi) fail(\"the range runs backwards: its end is before its start\");
    if (r.lo == r.hi) return;
    ll lo = (ll)r.lo, hi = (ll)r.hi;
    auto it = segs.upper_bound(lo);
    if (it == segs.begin()) return;
    auto lit = std::prev(it);
    ll sl = lit->first;
    ll seg_hi = std::get<0>(lit->second);
    V cur_val = std::get<1>(lit->second);
    if (seg_hi >= hi) {
        on_old(r, cur_val);
        if (cur_val == val) return;
        ll new_hi = hi;
        if (seg_hi > hi) {
            it = segs.emplace_hint(it, hi, std::tuple<ll, V>{seg_hi, cur_val});
        } else if (it != segs.end() && it->first == hi && std::get<1>(it->second) == val) {
            new_hi = std::get<0>(it->second);
            it = segs.erase(it);
        }
        if (sl < lo) {
            std::get<0>(lit->second) = lo;
            segs.emplace_hint(it, lo, std::tuple<ll, V>{new_hi, val});
        } else if (lit != segs.begin() && std::get<1>(std::prev(lit)->second) == val) {
            std::get<0>(std::prev(lit)->second) = new_hi;
            segs.erase(lit);
        } else {
            lit->second = std::tuple<ll, V>{new_hi, val};
        }
        return;
    }
    on_old(Range<Pos>{(Pos)lo, (Pos)seg_hi}, cur_val);
    auto left_target = segs.end();
    if (sl < lo) {
        if (cur_val == val) left_target = lit;
        else std::get<0>(lit->second) = lo;
    } else if (lit != segs.begin() && std::get<1>(std::prev(lit)->second) == val) {
        left_target = std::prev(lit);
        segs.erase(lit);
    } else {
        left_target = lit;
    }
    ll new_hi = hi;
    while (it != segs.end() && it->first < hi) {
        ll pos = it->first;
        ll ohi = std::get<0>(it->second);
        V ov = std::move(std::get<1>(it->second));
        if (ohi <= hi) {
            on_old(Range<Pos>{(Pos)pos, (Pos)ohi}, ov);
            it = segs.erase(it);
        } else {
            on_old(Range<Pos>{(Pos)pos, (Pos)hi}, ov);
            if (ov == val) {
                new_hi = ohi;
                it = segs.erase(it);
            } else {
                it = segs.erase(it);
                it = segs.emplace_hint(it, hi, std::tuple<ll, V>{ohi, std::move(ov)});
            }
            break;
        }
    }
    if (it != segs.end() && it->first == new_hi && std::get<1>(it->second) == val) {
        new_hi = std::get<0>(it->second);
        it = segs.erase(it);
    }
    if (left_target != segs.end()) left_target->second = std::tuple<ll, V>{new_hi, val};
    else segs.emplace_hint(it, lo, std::tuple<ll, V>{new_hi, val});
}
",
    },
    Piece {
        name: "map_fold",
        needs: &[],
        code: "\
template <class M, class K, class V> void map_max(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else if (v > it->second) it->second = v; }
template <class M, class K, class V> void map_min(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else if (v < it->second) it->second = v; }
template <class M, class K, class V> void map_mul(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else it->second *= v; }
template <class M, class K, class V> void map_and(M& m, const K& k, const V& v) { auto it = m.find(k); if (it == m.end()) m.emplace(k, v); else it->second &= v; }
",
    },
];
