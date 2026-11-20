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

struct Psum2 {
    vector<vector<int>> psum;

    Psum2(const vector<vector<int>>& arr)
        : psum(sz(arr) + 1, vector<int>(sz(arr[0]) + 1)) {
        int n = sz(arr), m = sz(arr[0]);
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < m; j++) {
                psum[i + 1][j + 1] =
                    psum[i + 1][j] + psum[i][j + 1] - psum[i][j] + arr[i][j];
            }
        }
    }

    int query(int x1, int y1, int x2, int y2) const {
        return psum[x2][y2] - psum[x1][y2] - psum[x2][y1] + psum[x1][y1];
    }
};

vector<int> vmins(const vector<int>& arr, int kv) {
    vector<int> ans;
    deque<pair<int, int>> q;
    for (int i = 0; i < sz(arr); i++) {
        while (sz(q) && q.back().second >= arr[i]) {
            q.pop_back();
        }
        q.emplace_back(i, arr[i]);
        while (sz(q) && q.front().first <= i - kv) {
            q.pop_front();
        }
        if (i >= kv - 1) {
            ans.push_back(q.front().second);
        }
    }
    return ans;
}

void solve() {
    int n, m, ax, ay, bx, by;
    cin >> m >> n >> ay >> ax >> by >> bx;
    vector<vector<int>> arr(n, vector<int>(m));
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    Psum2 psum(arr);
    vector<vector<int>> rect(n - bx + 1, vector<int>(m - by + 1));
    for (int i = 0; i + bx <= n; i++) {
        for (int j = 0; j + by <= m; j++) {
            rect[i][j] = psum.query(i, j, i + bx, j + by);
        }
    }
    int vx = ax - bx - 1, vy = ay - by - 1;
    vector<vector<int>> tmp1;
    for (auto& a : rect) {
        tmp1.push_back(vmins(a, vy));
    }
    int pmins[n][m];
    for (int i = 0; i < sz(tmp1[0]); i++) {
        vector<int> tmp(sz(tmp1));
        for (int j = 0; j < sz(tmp1); j++) {
            tmp[j] = tmp1[j][i];
        }
        auto cv = vmins(tmp, vx);
        for (int j = 0; j < sz(cv); j++) {
            pmins[j][i] = cv[j];
        }
    }
    array<int, 3> ans {};
    for (int i = 0; i + ax <= n; i++) {
        for (int j = 0; j + ay <= m; j++) {
            ans = max(ans, array<int, 3> {psum.query(i, j, i + ax, j + ay) -
                                              pmins[i + 1][j + 1],
                                          i, j});
        }
    }
    auto print = [&](int x, int y) -> void {
        cout << y + 1 << " " << x + 1 << endl;
    };
    auto [_, x1, y1] = ans;
    dbg(x1, y1);
    print(x1, y1);
    ans = {int(1e9)};
    for (int i = x1 + 1; i + bx < x1 + ax; i++) {
        for (int j = y1 + 1; j + by < y1 + ay; j++) {
            dbg(i, j);
            ans = min(ans, array<int, 3> {rect[i][j], i, j});
        }
    }
    auto [_a, x2, y2] = ans;
    print(x2, y2);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}