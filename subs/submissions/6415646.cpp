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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

int solve(int xa, int xb, const vector<int>& arr) {
    int n = sz(arr), ans = 0;

    vector<int> targ(n);
    for (int i = 0; i < n; i++) {
        if (i % 2) {
            targ[i] = xb;
        } else {
            targ[i] = xa;
        }
    }

    for (int i = 0; i < n; i++) {
        ans += arr[i] != targ[i];
    }

    vector<int> nx(n);
    for (int i = 0; i < n; i++) {
        nx[i] = arr[i] == (targ[i] ^ xa ^ xb);
    }
    dbg(nx);

    for (int i = 0; i < n;) {
        for (; i < n && !nx[i]; i++)
            ;
        if (i == n) {
            break;
        }
        int ql = i;
        for (; i < n && nx[i]; i++)
            ;
        // ans += (i - ql) / 2;
    }

    return ans;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
        a--;
        assert(0 <= a && a < n);
    }

    map<pair<int, int>, int> mp;
    for (int i = 0; i + 1 < n; i++) {
        if (i && arr[i + 1] == arr[i - 1]) {
            continue;
        }
        int j;
        for (j = i + 2; j < n && arr[j] == arr[i + (j - i) % 2]; j++)
            ;
        int ca = arr[i], cb = arr[i + 1];
        if (i % 2 == 0) {
            swap(ca, cb);
        }
        mp[{ca, cb}] += (j - i) / 2;
    }

    int cnt[2][n + 1] {};
    for (int i = 0; i < n; i++) {
        cnt[i % 2][arr[i]]++;
    }

    int ans = n;
    for (auto& [k, v] : mp) {
        auto& [a, b] = k;
        if (a == b) {
            continue;
        }
        ans = min(ans, n - cnt[0][a] - cnt[1][b] + v);
    }
    vector<int> cgraph[n + 1];
    for (int i = 0; i < n; i++) {
        mp[{i, i}] = 0;
    }
    for (auto& [k, _v] : mp) {
        auto& [a, b] = k;
        cgraph[a].push_back(b);
    }
    multiset<int> cms;
    for (auto& a : cnt[1]) {
        cms.insert(a);
    }
    for (int i = 0; i <= n; i++) {
        for (auto& a : cgraph[i]) {
            cms.erase(cms.find(cnt[1][a]));
        }
        ans = min(ans, n - cnt[0][i] - *rbegin(cms));
        if (n - cnt[0][i] - *rbegin(cms) == 4) {
            dbg(i);
        }
        for (auto& a : cgraph[i]) {
            cms.insert(cnt[1][a]);
        }
    }
    // for (int a = 0; a <= n; a++) {
    //     for (int b = 0; b <= n; b++) {
    //         if (a == b) {
    //             continue;
    //         }
    //         // ans = min(ans, n - cnt[0][a] - cnt[1][b] + mp[{a, b}]);
    //         assert(solve(a, b, arr) == n - cnt[0][a] - cnt[1][b]);
    //         ans = min(ans, n - cnt[0][a] - cnt[1][b] + mp[{a, b}]);
    //     }
    // }
    cout << ans << endl;
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