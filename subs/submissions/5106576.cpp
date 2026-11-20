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

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    kv--;
    bool arr[n][m];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    int score[m] {};
    for (auto& a : arr) {
        for (int i = 0; i < m; i++) {
            score[i] += !a[i];
        }
    }
    array<int, 3> sarr[n];
    for (int i = 0; i < n; i++) {
        int cs = 0, cnt = 0;
        for (int j = 0; j < m; j++) {
            if (arr[i][j]) {
                cnt++;
                cs += score[j];
            }
        }
        sarr[i] = {-cs, -cnt, i};
    }
    sort(sarr, sarr + n);
    for (int i = 0; i < n; i++) {
        if (sarr[i][2] == kv) {
            cout << -sarr[i][0] << " " << i + 1 << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}