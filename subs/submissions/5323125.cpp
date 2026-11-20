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
    vector<int> graph[n];
    for (int i = 0; i < n; i++) {
        int m;
        cin >> m;
        for (int j = 0; j < m; j++) {
            int p;
            cin >> p;
            p--;
            graph[p].push_back(j);
        }
    }
    int ans = 1e9;
    for (int i = 0; i < n; i++) {
        queue<int> q;
        int dist[n];
        memset(dist, -1, sizeof(dist));
        auto push = [&](int u, int d) -> void {
            if (dist[u] != -1) {
                return;
            }
            dist[u] = d;
            q.push(u);
        };
        push(i, 0);
        while (sz(q)) {
            int u = q.front();
            q.pop();
            for (auto& v : graph[u]) {
                push(v, dist[u] + 1);
            }
        }
        for (auto& a : dist) {
            if (a == -1) {
                goto loop;
            }
        }
        ans = min(ans, accumulate(dist, dist + n, 0));
    loop:;
    }
    cout << ans + n << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}