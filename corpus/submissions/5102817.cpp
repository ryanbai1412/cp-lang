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
    int n;
    cin >> n;
    int iarr[n];
    for (auto& a : iarr) {
        cin >> a;
    }
    int arr[n][2];
    for (int i = 0; i < n; i++) {
        auto& [u, v] = arr[i];
        int x = iarr[i];
        u = (i - x + n) % n;
        v = (i + x) % n;
    }
    vector<pair<int, int>> graph[n];
    int deg[n] {};
    for (int i = 0; i < n; i++) {
        auto& [u, v] = arr[i];
        dbg(u, v, i);
        deg[u]++;
        deg[v]++;
        graph[u].emplace_back(v, i);
        graph[v].emplace_back(u, i);
    }
    int ans[n];
    memset(ans, -1, sizeof(ans));
    bool visi[n] {}, viso[n] {};
    vector<int> q;
    auto set_eq = [&](int i, int u) -> void {
        dbg(i, u);
        if (!(!visi[i] && !viso[u])) {
            cout << "No Answer" << endl;
            exit(0);
        }
        visi[i] = viso[u] = true;
        ans[i] = u;
    };
    auto dfs = [&]() -> void {
        while (sz(q)) {
            int u = q.back();
            q.pop_back();
//            assert(deg[u] == 1);
            for (auto& [v, i] : graph[u]) {
                if (!visi[i]) {
                    set_eq(i, u);
                    deg[u]--;
                    deg[v]--;
                    if (deg[v] == 1) {
                        q.push_back(v);
                    }
                }
            }
        }
    };
    for (int i = 0; i < n; i++) {
        if (deg[i] == 1) {
            q.push_back(i);
        }
    }
    dfs();
    for (int i = 0; i < n; i++) {
        if (!visi[i]) {
            auto [u, v] = arr[i];
            int cu = min(u, v);
            set_eq(i, cu);
            cu ^= u ^ v;
            do {
                for (auto& [nv, ni] : graph[cu]) {
                    if (!visi[ni]) {
                        set_eq(ni, cu);
                        cu = nv;
                    }
                }
            } while (cu != min(u, v));
        }
    }
    if (any_of(visi, visi + n,
               [](bool b) {
                   return !b;
               }) ||
        any_of(viso, viso + n, [](bool b) {
            return !b;
        })) {
        cout << "No Answer" << endl;
        return;
    }
    for (int i = 0; i < n; i++) {
        cout << ans[i] << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}