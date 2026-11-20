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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

vector<ld> solve_mat(vector<vector<ld>> arr) {
    int n = sz(arr);
    for (int i = 0; i < n; i++) {
        assert(arr[i][i] != 0);
        for (int j = i + 1; j < n; j++) {
            ld cmul = arr[j][i] / arr[i][i];
            for (int k = 0; k <= n; k++) {
                arr[j][k] -= cmul * arr[i][k];
            }
        }
    }
    for (int i = n - 1; i >= 0; i--) {
        ld cdiv = arr[i][i];
        for (auto& a : arr[i]) {
            a /= cdiv;
        }
        for (int j = 0; j < i; j++) {
            ld cmul = arr[j][i];
            for (int k = 0; k <= n; k++) {
                arr[j][k] -= cmul * arr[i][k];
            }
        }
    }
    vector<ld> ans;
    for (auto& a : arr) {
        ans.push_back(a[n]);
    }
    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;
    string arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    map<string, int> conv;
    int cind = 0;
    {
        for (auto& a : arr) {
            for (int i = 0; i <= m; i++) {
                auto [it, inserted] = conv.emplace(a.substr(0, i), 0);
                if (inserted) {
                    it->second = cind++;
                }
            }
        }
    }
    auto to_pref = [&](const string& s) -> int {
        for (int i = 0;; i++) {
            auto it = conv.find(s.substr(i));
            if (it != conv.end()) {
                return it->second;
            }
        }
    };
    vector<string> arrs(cind);
    for (auto& [k, v] : conv) {
        arrs[v] = k;
    }
    vector<int> graph[cind];
    for (int i = 0; i < sz(arrs); i++) {
        if (sz(arrs[i]) == m) {
            continue;
        }
        for (char c : {'T', 'H'}) {
            int ni = to_pref(arrs[i] + c);
            graph[i].push_back(ni);
        }
    }
    ld dp[cind] {};
    dp[to_pref("")] = 1;
    for (int _iter = 0; _iter < 100000; _iter++) {
        ld ndp[cind] {};
        for (int i = 0; i < cind; i++) {
            if (sz(arrs[i]) == m) {
                ndp[i] += 2 * dp[i];
            }
            for (auto& v : graph[i]) {
                ndp[v] += dp[i];
            }
        }
        memcpy(dp, ndp, sizeof(dp));
    }
    vector<ld> ans;
    for (auto& a : arr) {
        ans.push_back(dp[to_pref(a)]);
    }
    ld sans = accumulate(begin(ans), end(ans), ld(0));
    for (auto& a : ans) {
        a /= sans;
    }
    for (auto& a : ans) {
        cout << a << endl;
    }
    /*vector<vector<ld>> mat(cind, vector<ld>(cind + 1));
    for (int i = 0; i < cind; i++) {
        mat[i][i] = -1;
    }
    for (int i = 0; i < n; i++) {
        for (auto& v : graph[i]) {
            mat[v][i] += 0.5;
        }
    }
    mat[to_pref("")][cind] = -1;
    auto ans = solve_mat(mat);
    for (auto& a : arr) {
        cout << ans[to_pref(a)] << endl;
    }*/
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(6);
    solve();
}