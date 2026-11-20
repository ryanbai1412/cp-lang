// cf bits/extc++.h nonsense
#ifdef ONLINE_JUDGE
#define _EXT_CODECVT_SPECIALIZATIONS_H 1
#define _EXT_ENC_FILEBUF_H 1
#endif
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

/*
Algorithm:

sweep left to right
find the first position at which there are no longer occurrences of all k
elements to the right of it find the smallest element to the left of that
position now repeat, ignoring that element
*/

void solve() {
    int n, kv;
    cin >> n >> kv;

    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    vector<int> last_occ(kv);
    for (int i = 0; i < n; i++) {
        last_occ[arr[i]] = i;
    }

    vector<pair<int, int>> locc;
    for (int i = 0; i < kv; i++) {
        locc.emplace_back(last_occ[i], i);
    }
    sort(begin(locc), end(locc));

    vector<bool> vis(kv);
    vector<int> ans;
    int cl = 0;

    for (int i = 0; i < sz(locc);) {
        auto& [x, v] = locc[i];

        if (vis[v]) {
            i++;
            continue;
        }

        int mn = 1e9;
        for (; cl <= x; cl++) {
            mn = min(mn, arr[cl]);
        }

        dbg(x, v, mn, cl);
        ans.push_back(mn);
        vis[mn] = true;
    }

    dbg(sz(ans));
    for (int i = 0; i < kv; i++) {
        cout << ans[i] + 1 << " \n"[i == kv - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}