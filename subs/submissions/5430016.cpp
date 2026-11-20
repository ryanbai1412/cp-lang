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
    int m;
    cin >> m;
    int arr[256][2] {};
    for (int i = 0; i < m; i++) {
        char c;
        cin >> c;
        cin >> arr[c][0] >> arr[c][1];
    }
    int n;
    cin >> n;
    int ans = 0;
    while (n--) {
        string s;
        cin >> s;
        int cnt[256] {};
        for (auto& a : s) {
            cnt[a]++;
        }
        int cans = 0;
        for (auto& a : s) {
            if (cnt[a] > arr[a][1]) {
                goto loop;
            }
            cans += arr[a][0];
        }
        ans = max(ans, cans);
    loop:;
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}