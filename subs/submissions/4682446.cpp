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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
    int n, m, k;
    cin >> n >> m >> k;
    int arr[k];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int dist[m][m], mp[n][n];
    memset(dist, 0x3f, sizeof(dist));
    memset(mp, -1, sizeof(mp));
    vector<int> cadj[n];
    for (int i = 0; i < m; i++) {
        int cn;
        cin >> cn;
        assert(cn >= 2);
        int carr[cn];
        for (auto& a : carr) {
            cin >> a;
            a--;
        }
        for (auto& a : carr) {
            cadj[a].push_back(i);
        }
        for (int j = 0; j < cn; j++) {
            int a = carr[j], b = carr[(j + 1) % cn];
            if (a > b) {
                swap(a, b);
            }
            if (mp[a][b] == -1) {
                mp[a][b] = i;
            } else {
                int u = mp[a][b];
                dist[i][u] = dist[u][i] = 1;
            }
        }
        dist[i][i] = 0;
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            for (int k = 0; k < n; k++) {
                dist[j][k] = min(dist[j][k], dist[j][i] + dist[i][k]);
            }
        }
    }
    pair<int, int> ans {1e9, 1e9};
    for (int i = 0; i < m; i++) {
        int cdist = 0;
        for (auto& a : arr) {
            int xdist = 1e9;
            for (auto& b : cadj[a]) {
                xdist = min(xdist, dist[i][b]);
            }
            cdist += xdist;
        }
        ans = min(ans, pair<int, int> {cdist, i});
    }
    cout << ans.first << endl;
    cout << ans.second + 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}