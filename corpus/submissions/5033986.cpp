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
    int n;
    cin >> n;
    string s;
    cin >> s;
    int dp[4][4][4][4] {};
    auto score = [&](int a, int b, int c) -> int {
        int ccnt = (a == 3) + (b == 3) + (c == 3);
        if (ccnt > 0) {
            if (ccnt == 1) {
                if (3 ^ a ^ b ^ c) {
                    return 2;
                }
                return 1;
            } else if (ccnt == 3) {
                return 0;
            }
            return 1;
        }
        if (a == b && a == c) {
            return 1;
        } else if (a != b && a != c && b != c) {
            return 3;
        } else {
            return 2;
        }
    };
    for (int i = n - 1; i >= 0; i--) {
        int ndp[4][4][4][4];
        memcpy(ndp, dp, sizeof(dp));
        int cx;
        if (s[i] == 'M') {
            cx = 0;
        } else if (s[i] == 'B') {
            cx = 1;
        } else {
            cx = 2;
        }
        for (int a1 = 0; a1 < 4; a1++) {
            for (int a2 = 0; a2 < 4; a2++) {
                for (int b1 = 0; b1 < 4; b1++) {
                    for (int b2 = 0; b2 < 4; b2++) {
                        dp[a1][a2][b1][b2] =
                            max(score(a1, a2, cx) + ndp[a2][cx][b1][b2],
                                score(b1, b2, cx) + ndp[a1][a2][b2][cx]);
                    }
                }
            }
        }
    }
    cout << dp[3][3][3][3] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}