#pragma GCC optimize("Ofast")

#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using u32 = uint32_t;
using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
void sort_unique(T& arr) {
    sort(begin(arr), end(arr));
    arr.erase(unique(begin(arr), end(arr)), arr.end());
}

template <int MOD>
struct Mint {
    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    int x;

    constexpr Mint() : x(0) {}
    constexpr Mint(long x) : x(norm(x)) {}

    static constexpr Mint from_unchecked(int x) {
        Mint m;
        m.x = x;
        return m;
    }

    friend ostream& operator<<(ostream& out, const Mint& m) {
        return out << m.x;
    }

    constexpr Mint operator-() const {
        if (!x) {
            return {};
        }
        return Mint::from_unchecked(MOD - x);
    }

    constexpr Mint& operator+=(const Mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr Mint& operator-=(const Mint& m) {
        return *this += -m;
    }

    constexpr Mint& operator*=(const Mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    constexpr friend Mint operator+(Mint a, const Mint& b) {
        return a += b;
    }
    constexpr friend Mint operator-(Mint a, const Mint& b) {
        return a -= b;
    }
    constexpr friend Mint operator*(Mint a, const Mint& b) {
        return a *= b;
    }
};

constexpr int MAXK = 55;

template <typename T, int MAXN>
struct Pow {
    T v[MAXN];

    constexpr Pow(T base) : v {} {
        v[0] = 1;
        for (int i = 1; i < MAXN; i++) {
            v[i] = v[i - 1] * base;
        }
    }

    constexpr const T& operator[](int ind) const {
        return v[ind];
    }
};

template <typename T, int BASE>
struct Hash {
    static constexpr Pow<T, MAXK> pow {BASE};

    int len;
    T x;

    Hash() : len(0), x() {}
    Hash(int x) : len(1), x(x) {}
    Hash(char c) : Hash(c - '0') {}
    Hash(int len, T x) : len(len), x(x) {}

    Hash operator+(const Hash& h) const {
        return {len + h.len, x * pow[h.len] + h.x};
    }

    Hash sub_pref(const Hash& h) const {
        return {len - h.len, x - pow[len - h.len] * h.x};
    }

    explicit operator u32() const {
        return x.x;
    }
};

template <typename A, typename B>
struct HashMult {
    A a;
    B b;

    HashMult() : a(), b() {}
    HashMult(int x) : a(x), b(x) {}
    HashMult(char c) : a(c), b(c) {}
    HashMult(const A& a, const B& b) : a(a), b(b) {}

    HashMult operator+(const HashMult& h) const {
        return {a + h.a, b + h.b};
    }

    HashMult sub_pref(const HashMult& h) const {
        return {a.sub_pref(h.a), b.sub_pref(h.b)};
    }
};

constexpr int MOD = 998244353;

using mint = Mint<MOD>;
// using H = HashMult<Hash<mint, 1412>, Hash<mint, 2141>>;

// u64 to_u64(const H& h) {
//     return (u64(u32(h.a)) << 32) | u32(h.b);
// }

using H = Hash<u64, 2141>;

u64 to_u64(const H& h) {
    return h.x;
}

vector<u64> part_hash(const string& s, int kv) {
    assert(kv <= sz(s));

    vector<u64> ans;

    H h;

    for (int i = 0; i < kv; i++) {
        h = h + s[i];
    }

    ans.push_back(to_u64(h));
    for (int i = kv; i < sz(s); i++) {
        h = h.sub_pref(s[i - kv]) + s[i];

        ans.push_back(to_u64(h));
    }

    return ans;
}

struct Node {
    int v;
    Node *l, *r;
};

struct HT {
    // map<u64, int> keys;
    // HT(const vector<u64>&) {}
    // int& operator[](u64 x) {
    //     return keys[x];
    // }
    // vector<u64> keys;
    // vector<int> vals;
    // int ignore = 0;

    // HT(const vector<u64>& _keys) : keys(_keys), vals(sz(keys)) {
    //     sort_unique(keys);
    // }

    // int& operator[](u64 x) {
    //     auto it = lower_bound(begin(keys), end(keys), x);

    //     if (it != keys.end() && *it == x) {
    //         return vals[it - keys.begin()];
    //     }
    //     return ignore;
    // }

    int n, ignore = 0;
    vector<u64> keys;
    vector<int> vals;

    int to_ind(u64 x) {
        x ^= x >> 33;
        x ^= x << 11;
        x ^= x >> 22;
        return x & (n - 1);
    }

    template <bool INSERT>
    int& find(u64 k) {
        for (int i = to_ind(k);; i = (i + 1) & (n - 1)) {
            if (keys[i] == -1) {
                if constexpr (INSERT) {
                    keys[i] = k;
                    return vals[i];
                } else {
                    return ignore;
                }
            }
            if (keys[i] == k) {
                return vals[i];
            }
        }
    }

    HT(const vector<u64>& init_keys)
        : n(1 << (__lg(sz(init_keys) + 10) + 2)), keys(n, -1), vals(n) {
        for (auto& a : init_keys) {
            find<true>(a);
        }
    }

    int& operator[](u64 x) {
        return find<false>(x);
    }
};

void solve() {
    int n, q;
    cin >> n >> q;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    vector<string> s_queries;
    vector<array<int, 3>> queries(q);

    for (auto& [ty, u, v] : queries) {
        cin >> ty;

        if (ty == 1) {
            cin >> u >> v;
            u--;
            v--;
        } else if (ty == 2) {
            cin >> u;
            u--;
        } else {
            string s;
            cin >> s >> v;
            u = sz(s_queries);
            s_queries.push_back(s);
        }
    }

    Node nodes[n] {};
    for (int i = 0; i < n; i++) {
        nodes[i].v = arr[i];
    }

    vector<HT> hts;
    vector<vector<u64>> s_part(sz(s_queries));

    for (int ckv = 0; ckv < MAXK; ckv++) {
        vector<u64> cvals;

        for (auto& [ty, sind, kv] : queries) {
            if (!(ty == 3 && kv == ckv)) {
                continue;
            }

            string& s = s_queries[sind];
            s_part[sind] = part_hash(s, kv);
            cvals.insert(cvals.end(), begin(s_part[sind]), end(s_part[sind]));
        }

        hts.emplace_back(cvals);
    }

    for (auto& a : arr) {
        hts[1][to_u64(H(a))]++;
    }

    auto update = [&](Node* u, Node* v, int inc) -> void {
        vector<H> u_suff, v_pref;

        {
            Node* o = u;

            u_suff.emplace_back();
            while (o && sz(u_suff) < MAXK) {
                u_suff.push_back(H(o->v) + u_suff.back());
                o = o->l;
            }
        }

        {
            Node* o = v;

            v_pref.emplace_back();
            while (o && sz(v_pref) < MAXK) {
                v_pref.push_back(v_pref.back() + H(o->v));
                o = o->r;
            }
        }

        for (int k = 1; k < MAXK; k++) {
            for (int ql = min(sz(u_suff) - 1, k - 1); ql >= 1; ql--) {
                int qr = k - ql;
                assert(qr >= 1);

                if (qr >= sz(v_pref)) {
                    break;
                }

                hts[k][to_u64(u_suff[ql] + v_pref[qr])] += inc;
            }
        }
    };

    auto do_merge = [&](Node* u, Node* v) -> void {
        update(u, v, +1);

        u->r = v;
        v->l = u;
    };

    auto do_split = [&](Node* u) -> void {
        Node* v = u->r;

        update(u, v, -1);

        u->r = nullptr;
        v->l = nullptr;
    };

    for (auto& [ty, u, v] : queries) {
        if (ty == 1) {
            do_merge(&nodes[u], &nodes[v]);
        } else if (ty == 2) {
            do_split(&nodes[u]);
        } else {
            mint ans = 1;

            int kv = v;

            for (auto& a : s_part[u]) {
                ans *= hts[kv][a];
            }

            cout << ans << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}