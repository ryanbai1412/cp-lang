#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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

bool vis[maxn];
int graph[maxn], tele[maxn], comp[maxn], vis_cnt;

void dfs(int u) {
    if (vis[u]) {
        return;
    }
    vis[u] = true;

    vis_cnt++;

    dfs(graph[u]);
}

int dfs_cnt(int u) {
    vis_cnt = 0;
    dfs(u);
    return vis_cnt;
}

void solve() {
    int n, kv;
    cin >> n >> kv;

    int arr[n][2];
    for (auto& [u, v] : arr) {
        cin >> u >> v;
    }

    memset(tele, -1, sizeof(tele));
    for (auto& [u, v] : arr) {
        tele[u] = v;
        tele[v] = u;
    }

    for (int i = 1; i < maxn; i++) {
        if (tele[i] != -1) {
            comp[i] = comp[i - 1] + 1;
        } else {
            comp[i] = comp[i - 1];
        }
    }

    for (auto& [u, v] : arr) {
        graph[comp[u - 1]] = comp[v];
        graph[comp[v - 1]] = comp[u];
    }

    int ans = dfs_cnt(0) - 1;

    vector<int> cycs;

    for (auto& a : comp) {
        if (vis[a]) {
            continue;
        }

        cycs.push_back(dfs_cnt(a));
    }

    if (kv < sz(cycs)) {
        nth_element(begin(cycs), cycs.end() - kv, end(cycs));
    }
    // sort(begin(cycs), end(cycs));

    for (int i = 0; i < kv; i++) {
        if (!sz(cycs)) {
            ans++;
            cycs.push_back(1);
            continue;
        }

        ans += 2 + cycs.back();
        cycs.pop_back();
    }

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}