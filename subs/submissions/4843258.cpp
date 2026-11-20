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

int mod;

struct ST {
    int n;
    vector<int> v;

    ST(int n) : n(n), v(4 * n, 1) {}

    void update(int o, int l, int r, int ind, int x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = v[lc] * v[rc] % mod;
    }

    void update(int ind, int x) {
        update(1, 0, n - 1, ind, (x + 1) % mod);
    }

    int query() const {
        return v[1];
    }
};

void solve() {
    int n, m;
    cin >> n >> m >> mod;
    pair<int, int> arr[n];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
        y--;
    }
    sort(arr, arr + n);
    vector<int> queries[n];
    {
        bool vis[n] {};
        for (int i = n - 1; i >= 0; i--) {
            auto& [len, t] = arr[i];
            if (!vis[t]) {
                vis[t] = true;
                queries[lower_bound(arr, arr + n,
                                    pair<int, int> {len / 2 + 1, -1}) -
                        arr]
                    .push_back(t);
            }
        }
    }
    bool ok[m] {};
    int cnt[m] {};
    ST st(m);
    long ans = 0;
    for (int i = 0; i < n; i++) {
        for (auto& a : queries[i]) {
            ok[a] = true;
            st.update(a, cnt[a]);
            ans += st.query();
        }
        int t = arr[i].second;
        cnt[t]++;
        if (ok[t]) {
            st.update(t, cnt[t]);
        }
    }
    cout << ans % mod << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}