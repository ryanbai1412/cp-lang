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
    int n, m, w;
    cin >> n >> m >> w;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    int targ[n];
    memcpy(targ, arr, sizeof(arr));
    sort(targ, targ + n);
    int ind[m];
    for (int i = 0; i < m; i++) {
        ind[i] = lower_bound(targ, targ + n, i) - targ;
    }
    auto nxt = [&](int x) -> int {
        dbg(x);
        int& i = ind[x];
        for (; arr[i] == x; i++)
            ;
        return i;
    };
    vector<vector<pair<int, int>>> ans;
    for (int i = 0; i < n;) {
        vector<pair<int, int>> cur;
        while (i < n && sz(cur) + 2 <= w) {
            for (; i < n && arr[i] == targ[i]; i++)
                ;
            if (i == n) {
                break;
            }
            int j = i, x = arr[i];
            while (x != targ[i] && sz(cur) + 2 <= w) {
                dbg(j, x);
                int k = nxt(x);
                cur.emplace_back(j, k);
                swap(arr[k], x);
                j = k;
            }
            cur.emplace_back(j, i);
            arr[i] = x;
        }
        if (sz(cur)) {
            ans.push_back(cur);
        }
    }
    cout << sz(ans) << endl;
    for (auto& a : ans) {
        cout << sz(a);
        for (auto& [x, y] : a) {
            cout << " " << x + 1 << " " << y + 1;
        }
        cout << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}