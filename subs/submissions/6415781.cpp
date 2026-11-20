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

namespace s1 {

void solve(int n, const vector<array<int, 4>>& edges) {
    int m = sz(edges);

    vector<pair<int, int>> segs;
    for (auto& [_u, _v, ql, qr] : edges) {
        segs.emplace_back(ql, qr);
    }

    set<pair<int, int>> cs;
    vector<int> evts[m];
    for (int i = 0; i < m; i++) {
        evts[segs[i].first].push_back(i);
    }

    int arr[m];
    for (int i = 0; i < m; i++) {
        for (auto& a : evts[i]) {
            cs.emplace(segs[a].second, a);
        }

        if (!sz(cs) || cs.begin()->first < i) {
            cout << "NO" << endl;
            return;
        }

        arr[cs.begin()->second] = i;
        cs.erase(cs.begin());
    }

    cout << "YES" << endl;
    for (int i = 0; i < m; i++) {
        cout << arr[i] + 1 << " \n"[i == m - 1];
    }
}

}  // namespace s1

void solve() {
    int n, m;
    cin >> n >> m;

    vector<array<int, 4>> edges(m);
    for (auto& [u, v, ql, qr] : edges) {
        cin >> u >> v >> ql >> qr;
        u--;
        v--;
        ql--;
        qr--;
    }

    if (m == n - 1) {
        s1::solve(n, edges);
        return;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}