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

using P = pair<int, int>;

P& operator+=(P& a, const P& b) {
    a.first += b.first;
    a.second += b.second;
    return a;
}

P operator+(P a, const P& b) {
    return a += b;
}

constexpr P DIRS[8] {{0, -1}, {1, -1}, {1, 0},  {1, 1},
                     {0, 1},  {-1, 1}, {-1, 0}, {-1, -1}};

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct Cmp {
    Cb cb;

    Cmp(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

alignas(16) char a_buf[1000 << 20];
size_t a_n = sizeof(a_buf);

template <typename T>
struct A {
    using value_type = T;

    T* allocate(size_t n) {
        a_n -= n * sizeof(T);
        a_n &= -alignof(T);
        return reinterpret_cast<T*>(a_buf + a_n);
    }

    void deallocate(T*, size_t) {}
};

template <typename T>
using AV = vector<T, A<T>>;

template <typename T, typename Adj, typename Id>
int dijkstra(int n,
             const vector<T>& srcs,
             const vector<T>& dests,
             Adj adj,
             Id id) {
    int dist[n];
    memset(dist, 0x3f, sizeof(dist));

    Cmp cmp([&](const pair<int, T>& p) -> int { return -p.first; });
    priority_queue<pair<int, T>, vector<pair<int, T>>, decltype(cmp)> pq(cmp);

    auto push = [&](T u, int d) -> void {
        int ui = id(u);
        dbg(u.pos, n);
        if (d >= dist[ui]) {
            return;
        }
        dist[ui] = d;
        pq.emplace(d, u);
    };

    for (auto& a : srcs) {
        push(a, 0);
    }

    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        if (dist[id(u)] != d) {
            continue;
        }

        adj(u, [&](const T& v, int w) -> void { push(v, d + w); });
    }

    int ans = 1e9;
    for (auto& a : dests) {
        ans = min(ans, dist[id(a)]);
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    n++;

    auto ibs = [&](const P& p) -> bool {
        auto [x, y] = p;
        return 0 <= x && x < n && 0 <= y && y < n;
    };

    vector<vector<array<int, 4>>> arr(n, vector<array<int, 4>>(n));

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n - 1; j++) {
            cin >> arr[i][j][2];
        }
    }
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n; j++) {
            cin >> arr[i][j][1];
        }
    }
    for (int i = 0; i < n; i++) {
        for (int j = 1; j < n; j++) {
            cin >> arr[i][j][0];
        }
    }
    for (int i = 1; i < n; i++) {
        for (int j = 0; j < n; j++) {
            cin >> arr[i][j][3];
        }
    }

    struct State {
        P pos;
        int dir;
    };

    vector<tuple<State, State, int>> edges;

    P sink {n - 1, n - 1};

    auto get_adj = [&](const State& s, auto cb) -> void {
        auto [p, d] = s;

        if (p == sink || !ibs(p)) {
            return;
        }

        int cadd = 0;

        for (int dd = 1; dd < 7; dd++) {
            int cd = (d + dd) % 8;

            if (dd > 1 && p + DIRS[cd] != sink) {
                cb(State {p + DIRS[cd], cd ^ 4}, cadd);
            }

            if (cd % 2 == 0) {
                cadd += arr[p.first][p.second][cd >> 1];
            }
        }
    };

    // for (int i = 0; i < n; i++) {
    //     for (int j = 0; j < n; j++) {
    //         P p {i, j};

    //         if (p == sink) {
    //             continue;
    //         }

    //         for (int d = 0; d < 8; d++) {
    //             get_adj({p, d}, [&](const State& v, int w) -> void {
    //                 edges.emplace_back(State {p, d}, v, w);
    //             });
    //         }
    //     }
    // }

    vector<State> srcs, sinks;
    for (int i = 0; i < n; i++) {
        srcs.push_back({{i, 0}, 0});
        srcs.push_back({{n - 1, i}, 2});
        sinks.push_back({{-1, i}, 2});
        sinks.push_back({{i, n}, 0});
    }

    auto id = [&](const State& s) -> int {
        return ((s.pos.first + 1) * (n + 1) + (s.pos.second + 1)) * 8 + s.dir;
    };

    // vector<tuple<int, int, int>> c_edges;
    // for (auto& [u, v, w] : edges) {
    //     c_edges.emplace_back(id(u), id(v), w);
    // }

    // vector<int> c_srcs, c_sinks;
    // for (auto& a : srcs) {
    //     c_srcs.push_back(id(a));
    // }
    // for (auto& a : sinks) {
    //     c_sinks.push_back(id(a));
    // }

    // dbg(sz(c_edges));
    cout << dijkstra((n + 2) * (n + 2) * 8, srcs, sinks, get_adj, id) << endl;
    // cout << dijkstra(c_edges, c_srcs, c_sinks) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}