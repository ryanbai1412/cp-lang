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

using ll = long long;
using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(const Cb& cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <typename T>
struct Comp {
    map<T, int> mp;

    int operator()(const T& t) {
        auto [it, inserted] = mp.emplace(t, -1);
        if (inserted) {
            it->second = sz(mp) - 1;
        }
        return it->second;
    }
};

template <typename T, typename Cmp = less<T>>
vector<T> sorted(vector<T> arr, Cmp cmp = Cmp()) {
    sort(begin(arr), end(arr), cmp);
    return arr;
}

struct H {
    u64 x = 0x1587896987;

    H& operator()(u64 y) {
        x += 0x18961568715 + y;
        x *= 0x82576789;
        x ^= x >> 19;
        x ^= x << 31;
        x ^= x >> 25;
        return *this;
    }

    template <typename A, typename B>
    H& operator()(const pair<A, B>& y) {
        return (*this)(y.first)(y.second);
    }
};

struct CompH {
    static constexpr int maxn = 1 << 23;

    u64 ks[maxn];
    int ind, vs[maxn];

    int operator()(u64 k) {
        assert(k);
        int i = int(k & (maxn - 1));
        while (true) {
            if (!ks[i]) {
                ks[i] = k;
                return vs[i] = ind++;
            } else if (ks[i] == k) {
                return vs[i];
            }
            i = (i + 1) & (maxn - 1);
        }
    }
} C;

template <typename S>
long dijkstra(const vector<tuple<S, S, long>>& edges, S src, S dest) {
    dbg("A");
    int n = 2 * sz(edges) + 10;

    // vector<S> a_s {src, dest};
    // for (auto& [u, v, _w] : edges) {
    //     a_s.push_back(u);
    //     a_s.push_back(v);
    // }
    // sort(begin(a_s), end(a_s));
    // a_s.erase(unique(begin(a_s), end(a_s)), a_s.end());

    auto comp = [&](S s) -> int {
        return C(H()(s).x);
        // return mp.insert({H()(s).x, sz(mp)}).first->second;
        // return int(lower_bound(begin(a_s), end(a_s), s) - a_s.begin());
    };

    vector<pair<int, long>> graph[n];

    for (auto& [u, v, w] : edges) {
        int cu = comp(u), cv = comp(v);
        graph[cu].emplace_back(cv, w);
        graph[cv].emplace_back(cu, w);
    }

    long dist[n];
    memset(dist, 0x3f, sizeof(dist));

    min_pq<pair<long, int>> pq;
    auto push = [&](int u, long d) -> void {
        if (d >= dist[u]) {
            return;
        }
        dist[u] = d;
        pq.emplace(d, u);
    };

    int c_src = comp(src), c_dest = comp(dest);

    push(c_src, 0);

    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();

        if (u == c_dest) {
            return d;
        }
        if (d != dist[u]) {
            continue;
        }

        for (auto& [v, w] : graph[u]) {
            push(v, d + w);
        }
    }

    return -1;
}

namespace s2 {

template <typename Cb>
void add_edges(vector<array<int, 4>> arr, const Cb& cb) {
    for (auto& [_ql, qr, _i, _] : arr) {
        // [] -> [)
        qr++;
    }

    int n = sz(arr);

    map<pair<int, int>, int> mp;

    auto split = [&](int ind) -> void {
        auto it = mp.lower_bound({ind + 1, -1});
        if (it == mp.begin()) {
            return;
        }
        --it;
        auto [cql, cqr] = it->first;

        if (cql < ind && ind < cqr) {
            int cv = it->second;
            mp.erase(it);
            mp[{cql, ind}] = cv;
            mp[{ind, cqr}] = cv;
        }
    };

    for (auto& [ql, qr, qi, _] : arr) {
        split(ql);
        split(qr);

        for (auto it = mp.lower_bound({ql, -1}); it != mp.end();
             it = mp.erase(it)) {
            auto [cql, cqr] = it->first;
            if (qr <= cql) {
                break;
            }

            assert(ql <= cql && cqr <= qr);
            cb(qi, it->second);
        }

        mp[{ql, qr}] = qi;
    }
}

long solve(vector<pair<int, int>> arr,
           vector<array<int, 3>> walk,
           int src,
           int dest) {
    clock_t c_start = clock();
    map<int, int> x_to_h;
    for (auto& [x, h] : arr) {
        x_to_h[x] = h;
    }
    int src_h = x_to_h[src], dest_h = x_to_h[dest];
    assert(src_h && dest_h);

    int c_src = sz(walk), c_dest = sz(walk) + 1;
    walk.push_back({src, src, 0});
    walk.push_back({dest, dest, 0});

    using S = pair<int, int>;

    vector<tuple<S, S, long>> edges;

    auto add_edge = [&](S s1, S s2) -> void {
        if (s1 > s2) {
            swap(s1, s2);
        }
        auto& [x1, y1] = s1;
        auto& [x2, y2] = s2;
        assert(x1 == x2 || y1 == y2);
        // dbg(s1, s2);
        edges.emplace_back(s1, s2, abs(x1 - x2) + abs(y1 - y2));
    };

    vector<array<int, 4>> n_walk;
    for (int i = 0; i < sz(walk); i++) {
        auto& [ql, qr, qh] = walk[i];
        n_walk.push_back({ql, qr, i, qh});
    }

    vector<S> w_segs[sz(walk)];

    vector<array<int, 5>> queries;

    auto inter = [&](int l1, int r1, int l2, int r2) -> bool {
        if (l1 > l2) {
            swap(l1, l2);
            swap(r1, r2);
        }
        return l2 <= r1;
    };

    auto go = [&](int u, int v, int x) -> void {
        int h1 = walk[u][2], h2 = walk[v][2];
        w_segs[u].emplace_back(x, h1);
        w_segs[v].emplace_back(x, h2);

        add_edge(S {x, h1}, S {x, h2});
    };

    auto cb_edge = [&](int u, int v) -> void {
        auto [l1, r1, h1] = walk[u];
        auto [l2, r2, h2] = walk[v];
        if (!inter(l1, r1, l2, r2)) {
            return;
        }

        int cql = max(l1, l2), cqr = min(r1, r2);
        queries.push_back({cql, cqr, max(h1, h2), u, v});
    };

    for (int i = 0; i < sz(walk); i++) {
        cb_edge(i, c_src);
        cb_edge(i, c_dest);
    }

    add_edges(
        sorted(n_walk, CmpByKey([&](const auto& a) -> int { return a[3]; })),
        cb_edge);
    // add_edges(
    //     sorted(n_walk, CmpByKey([&](const auto& a) -> int { return -a[3];
    //     })), cb_edge);

    map<int, vector<pair<bool, int>>> mp;
    for (auto& [x, h] : arr) {
        mp[-h].emplace_back(true, x);
    }
    for (int i = 0; i < sz(queries); i++) {
        mp[-queries[i][2]].emplace_back(false, i);
    }

    set<int> pos;

    dbg(double(clock() - c_start) / CLOCKS_PER_SEC);
    for (auto& [nh, e] : mp) {
        for (auto& [ty, x] : e) {
            if (ty) {
                pos.insert(x);
            }
        }

        for (auto& [ty, qi] : e) {
            if (ty) {
                continue;
            }
            auto& [cql, cqr, _h, u, v] = queries[qi];

            vector<int> ccache;
            auto go_c = [&](int x) -> void {
                if (cql <= x && x <= cqr) {
                    ccache.push_back(x);
                }
            };
            auto f_nearest = [&](int base) -> void {
                auto it = pos.lower_bound(base);
                if (it != pos.end()) {
                    go_c(*it);
                }
                if (it != pos.begin()) {
                    go_c(*(--it));
                }
            };

            f_nearest(cql);
            f_nearest(cqr);
            f_nearest(src);
            f_nearest(dest);

            sort(begin(ccache), end(ccache));
            ccache.erase(unique(begin(ccache), end(ccache)), ccache.end());
            for (auto& a : ccache) {
                go(u, v, a);
            }
        }
    }

    dbg(double(clock() - c_start) / CLOCKS_PER_SEC);
    for (auto& a : w_segs) {
        sort(begin(a), end(a));
        a.erase(unique(begin(a), end(a)), a.end());

        for (int i = 0; i + 1 < sz(a); i++) {
            add_edge(a[i], a[i + 1]);
        }
    }

    // dbg(sz(edges));
    // sort(begin(edges), end(edges));
    // edges.erase(unique(begin(edges), end(edges)), edges.end());
    // __gnu_pbds::gp_hash_table<u64, __gnu_pbds::null_type> ht;

    // edges.erase(remove_if(begin(edges), end(edges),
    //                       [&](const tuple<S, S, long>& e) -> bool {
    //                           auto& [a, b, c] = e;
    //                           u64 key = H {}(a)(b)(c).x;
    //                           return !ht.insert(key).second;
    //                       }),
    //             edges.end());

    dbg(sz(edges));
    dbg(double(clock() - c_start) / CLOCKS_PER_SEC);
    long ans = dijkstra(edges, S {src, 0}, S {dest, 0});

    dbg(double(clock() - c_start) / CLOCKS_PER_SEC);
    return ans;
}

}  // namespace s2

ll min_distance(vector<int> arr_x,
                vector<int> arr_h,
                vector<int> walk_l,
                vector<int> walk_r,
                vector<int> walk_y,
                int src,
                int dest) {
    int n = sz(arr_x), m = sz(walk_l);

    vector<pair<int, int>> arr(n);
    for (int i = 0; i < n; i++) {
        arr[i] = {arr_x[i], arr_h[i]};
    }

    vector<array<int, 3>> walk(m);
    for (int i = 0; i < m; i++) {
        walk[i] = {arr_x[walk_l[i]], arr_x[walk_r[i]], walk_y[i]};
    }

    src = arr_x[src];
    dest = arr_x[dest];

    return s2::solve(arr, walk, src, dest);
}