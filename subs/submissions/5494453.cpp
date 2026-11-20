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
    int a, b, c;
    cin >> a >> b >> c;
    array<int, 3> tmp {a, b, c};
    sort(begin(tmp), end(tmp));
    a = tmp[0];
    b = tmp[1];
    c = tmp[2];
    auto to_rep = [&](int a, int b, int c) -> int {
        int cmn = 1 << (__builtin_ctz(a) + 1);
        if ((b - a) % cmn == 0) {
            return cmn;
        } else {
            return -1;
        }
    };
    int cx = to_rep(a, b, c);
    bool ans;
    if (cx == -1) {
        ans = true;
    } else {
        ans = (c - b) % cx;
    }
    if (ans) {
        cout << "NO" << endl;
    } else {
        cout << "YES" << endl;
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