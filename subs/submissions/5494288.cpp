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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int m;
    string s;
    cin >> m >> s;
    int n = sz(s), arr[n];
    for (int i = 0; i < n; i++) {
        arr[i] = s[i] - 'a';
    }
    if (m >= 20) {
        cout << "NO" << endl;
        return;
    }
    bool dpa[m][1 << m] {}, dpb[1 << m] {};
    dpb[0] = true;
    for (int cit = n - 1; cit >= 0; cit--) {
        int cx = arr[cit];
        if (cx >= m) {
            continue;
        }
        for (int i = 0; i < (1 << m); i++) {
            dpa[cx][i | (1 << cx)] |= dpb[i];
        }
        for (int i = 0; i < (1 << m); i++) {
            bool cans = true;
            for (int j = 0; j < m; j++) {
                cans &= !on(i, j) || dpa[j][i];
            }
            dpb[i] |= cans;
        }
    }
    if (dpb[(1 << m) - 1]) {
        cout << "YES" << endl;
    } else {
        cout << "NO" << endl;
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