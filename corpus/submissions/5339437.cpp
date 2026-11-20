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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int prv[n], prv2[n], nxt[n], nxt2[n];
    {
        int vis[m + 1];
        memset(vis, -1, sizeof(vis));
        for (int i = 0; i < n; i++) {
            prv[i] = exchange(vis[arr[i]], i);
            prv2[i] = vis[arr[i] + 1];
        }
    }
    {
        int vis[m + 1];
        memset(vis, -1, sizeof(vis));
        for (int i = n - 1; i >= 0; i--) {
            nxt[i] = exchange(vis[arr[i]], i);
            nxt2[i] = vis[arr[i] + 1];
        }
    }
    long dist[n];
    memset(dist, 0x3f, sizeof(dist));
    min_pq<pair<long, int>> pq;
    auto push = [&](int u, long d) -> void {
        if (d < dist[u]) {
            dist[u] = d;
            pq.emplace(d, u);
        }
    };
    for (int i = 0; i < n; i++) {
        if (!arr[i]) {
            push(i, i + 1);
        }
    }
    while (sz(pq)) {
        auto [d, u] = pq.top();
        pq.pop();
        if (dist[u] != d) {
            continue;
        }
        for (auto& v : {prv[u], nxt[u], prv2[u], nxt2[u]}) {
            if (v == -1) {
                continue;
            }
            push(v, d + abs(u - v));
        }
    }
    long ans = 1e18;
    for (int i = 0; i < n; i++) {
        if (arr[i] == m - 1) {
            ans = min(ans, dist[i]);
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}