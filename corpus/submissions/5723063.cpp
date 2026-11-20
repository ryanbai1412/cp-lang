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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

struct mint {
    static constexpr int MOD = 998244353;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    static constexpr mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    constexpr mint operator-() const {
        if (!x) {
            return {};
        } else {
            return mint::from_unchecked(MOD - x);
        }
    }

    constexpr mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr mint& operator-=(const mint& m) {
        return *this += -m;
    }

    friend constexpr mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend constexpr mint operator-(mint a, const mint& b) {
        return a -= b;
    }
};

constexpr long LINF = long(mint::MOD) * mint::MOD;

struct Arr {
    int pos = 0;
    mint sum = 0;
    vector<long> arr;

    Arr(int n) : arr(n, LINF) {}

    long get(int ind) const {
        return arr[ind];
    }

    void update(int ind, int mul) {
        pos += (arr[ind] != LINF) * mul;
        sum += arr[ind] * mul;
    }

    void set(int ind, long x) {
        update(ind, -1);
        arr[ind] = x;
        update(ind, +1);
    }
};

struct Solver {
    int logn, n, m;
    vector<int> w_par;
    vector<vector<pair<int, int>>> graph;

    mint ans = 0;
    Arr dist;
    vector<Arr> cache;

    void copy_subtree(int o, const Arr& src, Arr& dest) {
        if (o >= n) {
            return;
        }
        dest.set(o, src.get(o));
        copy_subtree(o * 2, src, dest);
        copy_subtree(o * 2 + 1, src, dest);
    }

    void dijkstra_subtree(int o, long init_w, Arr& dist) {
        min_pq<pair<long, int>> pq;

        auto push = [&](int u, long d) -> void {
            if (u < o || d >= dist.get(u)) {
                return;
            }
            dist.set(u, d);
            pq.emplace(d, u);
        };

        push(o, init_w);

        while (sz(pq)) {
            auto [d, u] = pq.top();
            pq.pop();

            push(u / 2, d + w_par[u]);

            for (auto& [v, w] : graph[u]) {
                push(v, d + w);
            }
        }
    }

    Solver(int logn,
           int m,
           const vector<int>& arr,
           const vector<tuple<int, int, int>>& edges)
        : logn(logn),
          n(1 << logn),
          m(m),
          w_par(arr),
          graph(n),
          dist(n),
          cache(logn, Arr(n)) {
        for (auto& [u, v, w] : edges) {
            graph[u].emplace_back(v, w);
        }

        dfs(1, 0);
    }

    void dfs(int o, long d_w) {
        int d = __lg(o);
        copy_subtree(o, dist, cache[d]);

        dijkstra_subtree(o, -d_w, dist);

        ans += dist.sum + d_w * dist.pos;
        // for (auto& a : dist) {
        //     if (a == INF) {
        //         continue;
        //     }
        //     ans += a + d_w;
        // }

        if (d + 1 < logn) {
            for (int c : {o * 2, o * 2 + 1}) {
                dfs(c, d_w + w_par[c]);
            }
        }

        copy_subtree(o, cache[d], dist);
    }
};

void solve() {
    int logn, m;
    cin >> logn >> m;

    vector<int> arr(1 << logn);
    for (int i = 2; i < (1 << logn); i++) {
        cin >> arr[i];
    }

    vector<tuple<int, int, int>> edges(m);
    for (auto& [u, v, w] : edges) {
        cin >> u >> v >> w;
    }

    cout << Solver(logn, m, arr, edges).ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}