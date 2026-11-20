#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 2e6 + 5;

vector<pair<int, int>> graph[maxn];

struct Farthest {
    pair<int, int> opt {-1, -1};

    template <typename Cb>
    void dfs(int u, int p, int d, const Cb& cb) {
        opt = max(opt, {d, u});
        for (auto& [v, w] : graph[u]) {
            if (v == p || cb(v)) {
                continue;
            }
            dfs(v, u, d + w, cb);
        }
    }

    static pair<int, int> solve(int u) {
        return solve(u, [&](int) { return false; });
    }

    template <typename Cb>
    static pair<int, int> solve(int u, const Cb& cb) {
        Farthest solver;
        solver.dfs(u, -1, 0, cb);
        return solver.opt;
    }
};

struct FindPath {
    int dest;
    vector<pair<int, int>> ans, st;

    FindPath(int dest) : dest(dest) {}

    void dfs(int u, int p) {
        if (u == dest) {
            ans = st;
            ans.emplace_back(u, -1);
            return;
        }
        for (auto& [v, w] : graph[u]) {
            if (v == p) {
                continue;
            }
            st.emplace_back(u, w);
            dfs(v, u);
            st.pop_back();
        }
    }

    static vector<pair<int, int>> solve(int u, int v) {
        FindPath solver(v);
        solver.dfs(u, -1);
        return solver.ans;
    }
};

struct DS {
    int l = 0, r = 0;
    deque<pair<int, int>> q;

    void push_back(int x) {
        while (sz(q) && q.back().second <= x) {
            q.pop_back();
        }
        q.emplace_back(r++, x);
    }

    void pop_front() {
        if (l++ == q.front().first) {
            q.pop_front();
        }
    }

    int query_max() const {
        return q.front().second;
    }
};

void solve() {
    int n, kv;
    cin >> n >> kv;
    for (int i = 0; i < n - 1; i++) {
        int u, v, w;
        cin >> u >> v >> w;
        u--;
        v--;
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    int du = Farthest::solve(0).second, dv = Farthest::solve(du).second;
    auto path = FindPath::solve(du, dv);
    bool is_diam[n] {};
    for (auto& [u, _w] : path) {
        is_diam[u] = true;
    }
    auto farthest_skip = [&](int u) -> bool { return is_diam[u]; };
    int ans = INT_MAX, psum[sz(path)];
    psum[0] = 0;
    for (int i = 0; i + 1 < sz(path); i++) {
        psum[i + 1] = psum[i] + path[i].second;
    }
    DS ds;
    for (int i = 0, j = 0, clen = 0; i < sz(path); i++) {
        while (j < sz(path) && clen <= kv) {
            auto& [u, ulen] = path[j++];
            ds.push_back(Farthest::solve(u, farthest_skip).first);
            clen += ulen;
        }
        ans = min(ans, max({ds.query_max(), psum[i],
                            psum[sz(path) - 1] - psum[j - 1]}));
        ds.pop_front();
        clen -= path[i].second;
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}