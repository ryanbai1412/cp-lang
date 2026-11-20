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

void solve() {
    mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
    int n, m;
    cin >> n >> m;
    vector<int> graph[n];
    for (int i = 0; i < m; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    int kv = 0;
    for (auto& a : graph) {
        kv = max(kv, sz(a));
    }
    if (kv % 2 == 0) {
        kv++;
    }
    int ans[n];
    memset(ans, -1, sizeof(ans));
    auto go = [&](int u) -> void {
        bool vis[kv] {};
        for (auto& v : graph[u]) {
            if (ans[v] != -1) {
                vis[ans[v]] = true;
            }
        }
        for (int i = 0; i < kv; i++) {
            if (!vis[i]) {
                ans[u] = i;
                return;
            }
        }
        assert(false);
    };
    {
        bool vis[n] {};
        vector<int> q;
        auto push = [&](int u) -> void {
            if (vis[u]) {
                return;
            }
            vis[u] = true;
            q.push_back(u);
        };
        for (int i = 0; i < n; i++) {
            if (sz(graph[i]) < kv) {
                push(i);
            }
        }
        while (sz(q)) {
            int u = q.back();
            q.pop_back();
            if (sz(graph[u]) == kv) {
                go(u);
            }
            for (auto& v : graph[u]) {
                push(v);
            }
        }
    }
    for (int i = 0; i < n; i++) {
        if (ans[i] == -1) {
            go(i);
        }
    }
    cout << kv << endl;
    for (auto& a : ans) {
        cout << a + 1 << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}