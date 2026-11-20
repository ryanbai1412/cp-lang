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

struct S {
    int n, m, ind, cx = 0, cy = 0;
    vector<vector<int>> arr;

    S(int n, int m) : n(n), m(m), ind(1), arr(n, vector<int>(m)) {
        arr[0][0] = 1;
    }

    void right(int cnt) {
        for (int iter = 0; iter < cnt; iter++) {
            cy++;
            arr[cx][cy] = ++ind;
        }
    }

    void left(int cnt) {
        for (int iter = 0; iter < cnt; iter++) {
            cy--;
            arr[cx][cy] = ++ind;
        }
    }

    void up(int cnt) {
        for (int iter = 0; iter < cnt; iter++) {
            cx--;
            arr[cx][cy] = ++ind;
        }
    }

    void down(int cnt) {
        for (int iter = 0; iter < cnt; iter++) {
            cx++;
            arr[cx][cy] = ++ind;
        }
    }
};

template <typename T>
vector<vector<T>> transposed(const vector<vector<T>>& arr) {
    int n = sz(arr), m = sz(arr[0]);
    vector<vector<T>> ans(m, vector<T>(n));

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            ans[j][i] = arr[i][j];
        }
    }

    return ans;
}

void solve_even(S& ans) {
    int n = ans.n, m = ans.m;

    assert(n % 2 == 0 && m > 1);
    ans.right(m - 1);
    for (int i = 3; i < n; i += 2) {
        ans.down(1);
        ans.left(m - 2);
        ans.down(1);
        ans.right(m - 2);
    }
    ans.down(1);
    ans.left(m - 1);
    ans.up(n - 2);
}

void solve_odd(S& ans) {
    int n = ans.n, m = ans.m;

    assert(n % 2 == 1 && m % 2 == 1);

    ans.right(m - 1);
    ans.down(n - 1);
    for (int i = m - 3; i > 1; i -= 2) {
        ans.left(1);
        ans.up(n - 2);
        ans.left(1);
        ans.down(n - 2);
    }
    ans.left(1);
    for (int i = n - 2; i >= 3; i -= 2) {
        ans.left(1);
        ans.up(1);
        ans.right(1);
        ans.up(1);
    }
    ans.left(1);
    ans.up(1);
    ans.right(1);
}

vector<vector<int>> solve_all(int n, int m) {
    if (n % 2 == 0) {
        S ans(n, m);
        solve_even(ans);
        return ans.arr;
    } else if (m % 2 == 0) {
        return transposed(solve_all(m, n));
    }
    S ans(n, m);
    solve_odd(ans);
    return ans.arr;
}

void solve() {
    int n, m;
    cin >> n >> m;
    dbg(n, m);

    if (n == 1 || m == 1) {
        cout << -1 << endl;
        return;
    }

    auto ans = solve_all(n, m);

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            cout << ans[i][j] << " \n"[j == m - 1];
        }
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