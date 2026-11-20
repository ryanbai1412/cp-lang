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
#define sz(x) int(size(x))

void solve() {
    cout << "GetNM" << endl;
    int cnt_n, cnt_m;
    cin >> cnt_n >> cnt_m;
    int n = cnt_n + cnt_m + 1;
    bool arr[n][2];
    auto mod = [&](int i) -> int {
        if (i < 0) {
            i += n;
        } else if (i >= n) {
            i -= n;
        }
        return i;
    };
    // team 0 = truth
    // team 1 = lies
    auto query = [&](int i, int j, int x) -> bool {
        cout << "Ask " << mod(i) + 1 << " " << mod(j) + 1 << " " << x << endl;
        bool ans;
        cin >> ans;
        return ans;
    };
    for (int i = 0; i < n; i++) {
        arr[i][1] = query(i, i + 1, 0);
        arr[i][0] = query(i, i - 1, 0);
    }
    int ref = -1;
    for (int i = 0; i < n; i++) {
        if (arr[i][1] != arr[mod(i + 1)][0]) {
            assert(ref == -1);
            ref = i;
        }
    }
    assert(ref != -1);
    int ans[n];
    ans[ref] = 2;
    for (int i = ref - 1; i >= 0; i--) {
        ans[i] = arr[i][1] != !ans[i + 1];
    }
    for (int i = ref + 1; i < n; i++) {
        ans[i] = arr[i][0] != !ans[i - 1];
    }
    for (int i = 0; i < n; i++) {
        cout << "Answer " << ans[i] << endl;
    }
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}