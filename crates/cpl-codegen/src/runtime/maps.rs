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
// Salted hash for HashMap / HashSet (defeats anti-hash tests). The salt is read once at start-up,
// so a hash pays no first-use check.
inline const uint64_t hash_salt = (uint64_t)std::chrono::steady_clock::now().time_since_epoch().count();
struct CpHash {
    static uint64_t splitmix64(uint64_t x) {
        x += 0x9e3779b97f4a7c15ULL;
        x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
        x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
        return x ^ (x >> 31);
    }
    static uint64_t salt() { return hash_salt; }
    // Every integer width, signed or not (`Hash61` is a `U64`), and every enum hash as their 64-bit
    // value times a secret odd multiplier: multiply-shift, whose top bits (the table's home) are
    // universal, and HashTable mixes it with its own seed again.
    template <class I, std::enable_if_t<std::is_integral_v<I> || std::is_enum_v<I>, int> = 0>
    size_t operator()(I x) const { return (size_t)((uint64_t)x * (salt() | 1)); }
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
    // A K-dimensional array (`cpl::Grid`): its sizes, then each cell in row-major order.
    template <class G> auto operator()(const G& g) const -> decltype(g.d.data(), g.n[0], size_t()) {
        uint64_t h = salt();
        for (size_t k : g.n) h = splitmix64(h ^ k);
        for (const auto& x : g.d) h = splitmix64(h ^ (uint64_t)(*this)(x));
        return (size_t)h;
    }
    // A row held in a struct (`cpl::Row`): the same hash as the list it stands for.
    template <class R> auto operator()(const R& r) const -> decltype(r.vec(), r.a.data(), size_t()) {
        uint64_t h = salt() ^ r.size();
        for (const auto& x : r) h = splitmix64(h ^ (uint64_t)(*this)(x));
        return (size_t)h;
    }
    // An optional, packed or not.
    template <class O> auto operator()(const O& o) const -> decltype(o.has_value(), size_t()) { return o ? (*this)(*o) : (size_t)splitmix64(salt()); }
    // A hash set: the same for any order of its elements, so equal sets hash equal.
    template <class S> auto operator()(const S& s) const -> decltype(s.count(*s.begin()), size_t()) { uint64_t h = salt() ^ s.size(); for (const auto& x : s) h += splitmix64((uint64_t)(*this)(x)); return (size_t)splitmix64(h); }
    template <class T, size_t I = 0> uint64_t hash_tuple(const T& t) const {
        // One field (`Hash61(v: U64)`): the field's own hash, already salted and mixed.
        if constexpr (I == 0 && std::tuple_size<T>::value == 1) return (uint64_t)(*this)(std::get<0>(t));
        else if constexpr (I == std::tuple_size<T>::value) return salt();
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
        name: "hashcore",
        needs: &["fail"],
        code: "\
// Hands the whole pages inside [p, p + bytes) back to the OS, so a rehash frees the old table behind it.
inline void release_pages(void* p, size_t bytes) {
#if defined(__linux__)
    uintptr_t a = ((uintptr_t)p + 4095) & ~(uintptr_t)4095, b = ((uintptr_t)p + bytes) & ~(uintptr_t)4095;
    if (b > a) ::madvise((void*)a, b - a, MADV_DONTNEED);
#else
    (void)p; (void)bytes;
#endif
}
// The fewest slots that hold n entries at a load of at most 3/4 (at least 8): linear probing past
// that walks long runs, most of all for a key that is absent.
inline size_t hash_cap(size_t n) { return std::max<size_t>(8, n + (n + 2) / 3); }
// The top 64 bits of a * b: maps a hash onto [0, cap) in order, for any cap.
inline uint64_t mulhi(uint64_t a, uint64_t b) {
#if defined(__SIZEOF_INT128__)
    return (uint64_t)(((unsigned __int128)a * b) >> 64);
#else
    uint64_t al = (uint32_t)a, ah = a >> 32, bl = (uint32_t)b, bh = b >> 32, m = ah * bl + ((al * bl) >> 32);
    return ah * bh + (m >> 32) + ((al * bh + (uint32_t)m) >> 32);
#endif
}
// Open addressing with linear probing over packed slots: a state byte (0 empty, else 0x80 and 7 bits
// of the hash) and the entry, built only in a full slot, so a probe reads an entry only when the
// fingerprint matches. The load stays at most 3/4 and `reserve` sizes exactly. An entry's home grows
// with its hash, so a rehash walks the old table in the new one's order and frees it as it goes.
// A fresh odd seed per table, so two tables put the same keys in unrelated orders: draining one
// into another in slot order would otherwise insert in home order and pile up one probe run.
// The seeds run from a fixed start, so a program's output does not change from run to run.
inline uint64_t table_seed() {
    static uint64_t c = 0x2545f4914f6cdd1dULL;
    uint64_t x = c += 0x9e3779b97f4a7c15ULL;
    x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL; x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
    return (x ^ (x >> 31)) | 1;
}
template <class E, class K, class KeyOf, class H> struct HashTable {
    uint8_t* st = nullptr; E* e = nullptr; size_t cap = 0, n = 0; uint64_t seed = table_seed();
    HashTable() = default;
    HashTable(const HashTable& o) { copy(o); }
    HashTable(HashTable&& o) noexcept { steal(o); }
    HashTable& operator=(const HashTable& o) { if (this != &o) { drop(); copy(o); } return *this; }
    HashTable& operator=(HashTable&& o) noexcept { if (this != &o) { drop(); steal(o); } return *this; }
    ~HashTable() { drop(); }
    void steal(HashTable& o) { st = o.st; e = o.e; cap = o.cap; n = o.n; seed = o.seed; o.st = nullptr; o.e = nullptr; o.cap = o.n = 0; }
    void alloc(size_t c) {
        st = (uint8_t*)calloc(c, 1); e = (E*)calloc(c, sizeof(E));
        if (!st || !e) fail(\"out of memory\");
        cap = c;
    }
    void copy(const HashTable& o) {
        if (!o.cap) return;
        seed = o.seed; alloc(o.cap);
        for (size_t i = 0; i < cap; ++i) if (o.st[i]) { new (e + i) E(o.e[i]); st[i] = o.st[i]; }
        n = o.n;
    }
    void kill(size_t i) { if constexpr (!std::is_trivially_destructible_v<E>) e[i].~E(); st[i] = 0; }
    void drop() {
        if (!cap) return;
        for (size_t i = 0; i < cap; ++i) if (st[i]) kill(i);
        free(st); free(e); st = nullptr; e = nullptr; cap = n = 0;
    }
    void clear() {
        if (!n) return;
        if (cap > 64 && n * 8 < cap) { drop(); return; }
        for (size_t i = 0; i < cap; ++i) if (st[i]) kill(i);
        n = 0;
    }
    // The salted hash keyed by the table's seed, mixed again so a 32-bit size_t still fills the top
    // bits the home is taken from.
    uint64_t hash(const K& k) const { return ((uint64_t)H{}(k) ^ seed) * 0x9e3779b97f4a7c15ULL; }
    // From the middle bits: the home takes the top ones, and a multiply leaves the low ones
    // depending only on the key's low bits.
    static uint8_t fp(uint64_t h) { return (uint8_t)(0x80 | ((h >> 32) & 0x7f)); }
    size_t home(uint64_t h) const { return (size_t)mulhi(h, cap); }
    size_t next(size_t i) const { return i + 1 == cap ? 0 : i + 1; }
    size_t locate(const K& k) const { return n ? locate(k, hash(k)) : cap; }
    size_t locate(const K& k, uint64_t h) const {
        if (!n) return cap;
        uint8_t f = fp(h);
        for (size_t i = home(h); st[i]; i = next(i)) if (st[i] == f && KeyOf{}(e[i]) == k) return i;
        return cap;
    }
    // Rehashes into c slots, walking the old slots in order and freeing each page once its entries are
    // moved out and destroyed, so the old and new tables are never both whole, whatever E is.
    void grow(size_t c) {
        HashTable t; t.seed = seed; t.alloc(c);
        const size_t chunk = 1 << 14; size_t done = 0;
        for (size_t i = 0; i < cap; ++i) {
            if (st[i]) { t.put(std::move(e[i])); kill(i); }
            if (i + 1 - done == chunk) {
                release_pages(st + done, chunk); release_pages(e + done, chunk * sizeof(E)); done = i + 1;
            }
        }
        n = 0; drop(); steal(t);
    }
    void reserve(size_t want) { size_t c = hash_cap(want); if (c > cap) grow(c); }
    // Puts an entry whose key is absent; returns its slot.
    size_t put(E x) { uint64_t h = hash(KeyOf{}(x)); return put(std::move(x), h); }
    size_t put(E x, uint64_t h) {
        if (cap < hash_cap(n + 1)) grow(std::max(hash_cap(n + 1), cap + cap / 2));
        size_t i = home(h);
        while (st[i]) i = next(i);
        new (e + i) E(std::move(x)); st[i] = fp(h); ++n; return i;
    }
    // The slot of key k, putting mk() there when k is absent, in one walk: {slot, put}.
    template <class Mk> std::pair<size_t, bool> slot(const K& k, uint64_t h, Mk&& mk) {
        if (cap < hash_cap(n + 1)) { size_t i = locate(k, h); if (i != cap) return {i, false}; return {put(mk(), h), true}; }
        uint8_t f = fp(h); size_t i = home(h);
        for (; st[i]; i = next(i)) if (st[i] == f && KeyOf{}(e[i]) == k) return {i, false};
        new (e + i) E(mk()); st[i] = f; ++n; return {i, true};
    }
    // Removes slot i, moving back each later entry of its run whose home allows it.
    void erase_at(size_t i) {
        auto gap = [&](size_t a, size_t b) { return b >= a ? b - a : b + cap - a; };
        for (size_t j = next(i); st[j]; j = next(j)) {
            if (gap(home(hash(KeyOf{}(e[j]))), j) >= gap(i, j)) { e[i] = std::move(e[j]); st[i] = st[j]; i = j; }
        }
        kill(i); --n;
    }
};
template <class E, bool C> struct SlotIter {
    using P = std::conditional_t<C, const E, E>;
    const uint8_t* s; P* p; size_t i, cap;
    void skip() { while (i < cap && !s[i]) ++i; }
    P& operator*() const { return p[i]; }
    P* operator->() const { return p + i; }
    SlotIter& operator++() { ++i; skip(); return *this; }
    bool operator==(const SlotIter& o) const { return i == o.i; }
    bool operator!=(const SlotIter& o) const { return i != o.i; }
};
",
    },
    Piece {
        name: "cphashmap",
        needs: &["cphash", "hashcore"],
        code: "\
template <class K, class V, class H = CpHash> struct HashMap {
    using key_type = K; using mapped_type = V; using value_type = std::pair<K, V>;
    struct First { const K& operator()(const value_type& x) const { return x.first; } };
    HashTable<value_type, K, First, H> t;
    using Iter = SlotIter<value_type, false>; using CIter = SlotIter<value_type, true>;
    Iter it(size_t i) { return {t.st, t.e, i, t.cap}; }
    CIter it(size_t i) const { return {t.st, t.e, i, t.cap}; }
    size_t size() const { return t.n; }
    bool empty() const { return t.n == 0; }
    void clear() { t.clear(); }
    void reserve(size_t n) { t.reserve(n); }
    Iter begin() { Iter i = it(0); i.skip(); return i; }
    Iter end() { return it(t.cap); }
    CIter begin() const { CIter i = it(0); i.skip(); return i; }
    CIter end() const { return it(t.cap); }
    Iter find(const K& k) { return it(t.locate(k)); }
    CIter find(const K& k) const { return it(t.locate(k)); }
    size_t count(const K& k) const { return t.locate(k) != t.cap ? 1 : 0; }
    V& at(const K& k) { size_t i = t.locate(k); if (i == t.cap) fail(\"key not in map\"); return t.e[i].second; }
    const V& at(const K& k) const { size_t i = t.locate(k); if (i == t.cap) fail(\"key not in map\"); return t.e[i].second; }
    V& operator[](const K& k) { size_t i = t.slot(k, t.hash(k), [&] { return value_type(k, V{}); }).first; return t.e[i].second; }
    std::pair<Iter, bool> emplace(const K& k, const V& v) {
        auto [i, put] = t.slot(k, t.hash(k), [&] { return value_type(k, v); });
        return {it(i), put};
    }
    void erase(Iter i) { t.erase_at(i.i); }
    size_t erase(const K& k) { size_t i = t.locate(k); if (i == t.cap) return 0; t.erase_at(i); return 1; }
    bool operator==(const HashMap& o) const {
        if (t.n != o.t.n) return false;
        for (const auto& [k, v] : *this) { size_t j = o.t.locate(k); if (j == o.t.cap || !(o.t.e[j].second == v)) return false; }
        return true;
    }
    bool operator!=(const HashMap& o) const { return !(*this == o); }
};
",
    },
    Piece {
        name: "cphashset",
        needs: &["cphash", "hashcore"],
        code: "\
template <class K, class H = CpHash> struct HashSet {
    using key_type = K; using value_type = K;
    struct Self { const K& operator()(const K& x) const { return x; } };
    HashTable<K, K, Self, H> t;
    using CIter = SlotIter<K, true>;
    CIter it(size_t i) const { return {t.st, t.e, i, t.cap}; }
    size_t size() const { return t.n; }
    bool empty() const { return t.n == 0; }
    void clear() { t.clear(); }
    void reserve(size_t n) { t.reserve(n); }
    CIter begin() const { CIter i = it(0); i.skip(); return i; }
    CIter end() const { return it(t.cap); }
    CIter find(const K& k) const { return it(t.locate(k)); }
    size_t count(const K& k) const { return t.locate(k) != t.cap ? 1 : 0; }
    std::pair<CIter, bool> insert(const K& k) {
        auto [i, put] = t.slot(k, t.hash(k), [&] { return K(k); });
        return {it(i), put};
    }
    size_t erase(const K& k) { size_t i = t.locate(k); if (i == t.cap) return 0; t.erase_at(i); return 1; }
    bool operator==(const HashSet& o) const {
        if (t.n != o.t.n) return false;
        for (const K& k : *this) if (!o.count(k)) return false;
        return true;
    }
    bool operator!=(const HashSet& o) const { return !(*this == o); }
};
template <bool Same = false, class K, class H> std::vector<K> sorted(const HashSet<K, H>& s) { std::vector<K> v; v.reserve(s.size()); for (const K& k : s) v.push_back(k); sort_seq<Same>(v.begin(), v.end()); return v; }
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
        name: "lent",
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
template <class V> Lent<V> lend(const V& v) { return {&v}; }
template <class V> Lent<V> lend_opt(const std::optional<V>& o) { return {o ? &*o : nullptr}; }
template <class V> Lent<V> lend_opt(Lent<V> l) { return l; }
",
    },
    Piece {
        name: "held",
        needs: &[],
        code: "\
template <class T> struct Held {
    T own;
    const T* p;
    const T& operator*() const { return p ? *p : own; }
    T take() && { return p ? T(*p) : std::move(own); }
};
template <class T> Held<T> held_ref(const T& v) { return {T(), &v}; }
template <class T> Held<T> held_own(T v) { return {std::move(v), nullptr}; }
",
    },
    Piece {
        name: "lend_niche",
        needs: &["lent"],
        code: "\
// Any other optional layout, such as a niche `OptField`.
template <class O> auto lend_opt(const O& o) -> Lent<std::decay_t<decltype(*o)>> { return {o ? &*o : nullptr}; }
",
    },
    Piece {
        name: "map_lend",
        needs: &["lent"],
        code: "\
template <class M, class Q> Lent<typename M::mapped_type> map_lend(const M& m, const Q& k) {
    auto it = m.find(k); return {it == m.end() ? nullptr : &it->second};
}
",
    },
    Piece {
        name: "map_slot",
        needs: &["fail"],
        code: "\
template <class M, class Q> typename M::mapped_type* map_slot(M& m, const Q& k) {
    auto it = m.find(k); return it == m.end() ? nullptr : &it->second;
}
template <class V> V& map_slot_at(V* p) { if (!p) fail(\"key not in map\"); return *p; }
template <class M, class Q> typename M::mapped_type& map_slot_put(M& m, typename M::mapped_type*& p, const Q& k) {
    return p ? *p : *(p = &m[k]);
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
        name: "map_put_or",
        needs: &[],
        code: "\
template <class M, class Q, class F> typename M::mapped_type& map_put_or(M& m, const Q& k, F d) {
    auto it = m.lower_bound(k); if (it == m.end() || m.key_comp()(k, it->first)) it = m.emplace_hint(it, k, d()); return it->second;
}
",
    },
    Piece {
        name: "hash_put_or",
        needs: &["cphashmap"],
        code: "\
template <class K, class V, class H, class F> V& hash_put_or(HashMap<K, V, H>& m, const K& k, F d) {
    size_t i = m.t.slot(k, m.t.hash(k), [&] { return std::pair<K, V>(k, d()); }).first; return m.t.e[i].second;
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
