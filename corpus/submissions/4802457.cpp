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

constexpr int maxn = 1e5 + 5;

int n, k, lims[maxn][2];
long ans;
vector<long> weights;
vector<int> arr;
vector<array<int, 3>> reqs2;
vector<pair<int, int>> reqs[maxn];

void dfs(int ind) {
    if (ind == n) {
        for (auto& [u, v, w] : reqs2) {
            if (abs(arr[u] - arr[v]) > w) {
                return;
            }
        }
        long cans = 0;
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                cans += long(1e6) * (abs(arr[i] - arr[j]) <= 1);
            }
        }
        for (auto& a : arr) {
            cans += weights[a];
        }
        assert(cans >= 0);
        ans = max(ans, cans);
        return;
    }
    for (int i = lims[ind][0]; i <= lims[ind][1]; i++) {
        //        for (auto& [v, w] : reqs[ind]) {
        //            if (abs(i - arr[v]) > w) {
        //                goto loop;
        //            }
        //        }
        arr.push_back(i);
        dfs(ind + 1);
        arr.pop_back();
    loop:;
    }
}

void solve() {
    int m, q;
    cin >> k >> n >> m >> q;
    for (int i = 0; i < n; i++) {
        cin >> lims[i][0] >> lims[i][1];
    }
    reqs2.resize(m);
    for (auto& [u, v, w] : reqs2) {
        cin >> u >> v >> w;
        u--;
        v--;
        dbg(u, v, w);
        if (u > v) {
            swap(u, v);
        }
        if (u != v) {
            reqs[v].emplace_back(u, w);
        }
    }
    while (q--) {
        weights = vector<long>(k + 5);
        for (int i = 2; i <= k - 1; i++) {
            cin >> weights[i];
        }
        ans = -1;
        arr.clear();
        dfs(0);
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tc_id, tcs;
    cin >> tc_id >> tcs;
    while (tcs--) {
        solve();
    }
}