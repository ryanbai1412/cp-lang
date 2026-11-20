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

template <typename T>
optional<T> transposed(const optional<T>& x) {
    if (!x) {
        return {};
    }
    return transposed(x.value());
}

vector<vector<char>> sg(int n, int m, int r, int c) {
    assert(0 < r && r < n && 0 < c && c < m);
    vector<vector<char>> ans(n, vector<char>(m));
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < m; j++) {
            ans[i][j] = "ba"[i < r || j < c];
        }
    }
    return ans;
}

vector<vector<char>> s0(int n, int m, int r, int c) {
    assert(!r);
    vector<vector<char>> ans(n, vector<char>(m, 'a'));
    if (!c) {
        ans[0][0] = 'a';
        ans[0].back() = 'b';
        ans.back()[0] = 'c';
        ans.back().back() = 'd';
        for (int i = 1; i + 1 < n; i++) {
            ans[i][0] = 'a';
            ans[i].back() = 'b';
        }
        for (int j = 1; j + 1 < m; j++) {
            ans[0][j] = 'a';
            ans.back()[j] = 'b';
        }
        return ans;
    } else if (c == m) {
        for (auto& a : ans) {
            fill(begin(a), end(a), 'a');
            a.back() = 'b';
        }
        return ans;
    }
    for (auto& a : ans) {
        fill(begin(a), end(a), 'a');
        a.back() = 'b';
    }
    for (int i = c; i < m; i++) {
        ans.back()[i] = 'c';
    }
    return ans;
}

optional<vector<vector<char>>> sn(int n, int m, int r, int c) {
    assert(r == n && c);
    vector<vector<char>> ans(n, vector<char>(m, 'a'));
    if (c == m) {
        return ans;
    }
    if (m % 2 == 0 && c % 2 == 1) {
        return {};
    }
    vector<int> cols;
    if (c % 2 == 1) {
        assert(m % 2 == 1);
        cols.push_back(m / 2);
        c--;
    }
    {
        int cl = 0, cr = m - 1;
        while (c) {
            cols.push_back(cl++);
            cols.push_back(cr--);
            c -= 2;
        }
    }
    fill(begin(ans.back()), end(ans.back()), 'b');
    for (auto& a : cols) {
        ans.back()[a] = 'a';
    }
    return ans;
}

void grade(int n, int m, int r, int c, const vector<vector<char>>& arr) {
    assert(sz(arr) == n && sz(arr[0]) == m);
    for (auto& a : arr) {
        for (auto& b : a) {
            assert('a' <= b && b <= 'z');
        }
    }
    int cr = 0, cc = 0;
    for (auto& a : arr) {
        for (int i = 0; i < m; i++) {
            if (a[i] != a[m - i - 1]) {
                goto loop1;
            }
        }
        cr++;
    loop1:;
    }
    for (int i = 0; i < m; i++) {
        for (int j = 0; j < n; j++) {
            if (arr[j][i] != arr[n - j - 1][i]) {
                goto loop2;
            }
        }
        cc++;
    loop2:;
    }
    assert(cr == r && cc == c);
}

void solve() {
    int n, m, r, c;
    cin >> n >> m >> r >> c;
    optional<vector<vector<char>>> ans;
    if (0 < r && r < n && 0 < c && c < m) {
        ans = sg(n, m, r, c);
    } else if (!r) {
        ans = s0(n, m, r, c);
    } else if (!c) {
        ans = transposed(s0(m, n, c, r));
    } else if (r == n) {
        ans = sn(n, m, r, c);
    } else if (c == m) {
        ans = transposed(sn(m, n, c, r));
    } else {
        assert(false);
    }
    if (!ans.has_value()) {
        cout << "IMPOSSIBLE" << endl;
        return;
    }
    auto qans = ans.value();
    for (auto& a : qans) {
        for (auto& b : a) {
            cout << b;
        }
        cout << endl;
    }
    cout.flush();
    grade(n, m, r, c, qans);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}