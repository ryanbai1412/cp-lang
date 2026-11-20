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

struct A {
    long mn, mx;

    A() : mn(1e18), mx(-1e18) {}
    A(long x) : mn(x), mx(x) {}
    A(long mn, long mx) : mn(mn), mx(mx) {}

    friend A operator+(const A& a, const A& b) {
        return {a.mn + b.mn, a.mx + b.mx};
    }

    friend A operator*(const A& a, const A& b) {
        A ans;
        for (auto& xa : {a.mn, a.mx}) {
            for (auto& xb : {b.mn, b.mx}) {
                ans.update(xa * xb);
            }
        }
        return ans;
    }

    void update(long x) {
        mn = min(mn, x);
        mx = max(mx, x);
    }

    void update(const A& a) {
        update(a.mn);
        update(a.mx);
    }
};

long solve(const vector<int>& arr, const vector<int>& ops) {
    int n = sz(arr);
    A dp[n][n];
    for (int i = n - 1; i >= 0; i--) {
        dp[i][i] = arr[i];
        for (int j = i + 1; j < n; j++) {
            for (int k = i; k < j; k++) {
                if (ops[k] == 0) {
                    dp[i][j].update(dp[i][k] + dp[k + 1][j]);
                } else {
                    dp[i][j].update(dp[i][k] * dp[k + 1][j]);
                }
            }
        }
    }
    return dp[0][n - 1].mx;
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n), ops(n);
    for (int i = 0; i < n; i++) {
        char c;
        int x;
        cin >> c >> x;
        ops[i] = c == 'x';
        arr[i] = x;
    }
    long ans[n];
    for (int i = 0; i < n; i++) {
        int x = ops[0];
        ops.erase(ops.begin());
        ans[i] = solve(arr, ops);
        ops.push_back(x);
        x = arr[0];
        arr.erase(arr.begin());
        arr.push_back(x);
    }
    long mx = *max_element(ans, ans + n);
    cout << mx << endl;
    vector<int> vans;
    for (int i = 0; i < n; i++) {
        if (ans[i] == mx) {
            vans.push_back(i);
        }
    }
    for (int i = 0; i < sz(vans); i++) {
        cout << vans[i] + 1 << " \n"[i == sz(vans) - 1];
    }
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}