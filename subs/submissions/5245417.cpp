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

struct DS1 {
    int n, m;
    vector<vector<int>> psum;

    DS1(const vector<vector<int>>& arr)
        : n(sz(arr)), m(sz(arr[0])), psum(n + 1, vector<int>(m + 1)) {
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

int solve(const vector<vector<int>>& arr) {
    int n = sz(arr);
    DS1 psum(arr);
    int ans = 0;
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            map<int, int> mp;
            for (int x = i; x < n; x++) {
                for (int y = j; y < n; y++) {
                    mp[psum.query(i, j, x + 1, y + 1)]++;
                }
            }
            for (int x = 0; x < i; x++) {
                for (int y = 0; y < j; y++) {
                    auto it = mp.find(psum.query(x, y, i, j));
                    if (it != mp.end()) {
                        ans += it->second;
                    }
                }
            }
        }
    }
    return ans;
}

template <typename T>
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

void solve() {
    int n;
    cin >> n;
    vector<vector<int>> arr(n, vector<int>(n));
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    cout << solve(arr) + solve(reversed(arr)) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}