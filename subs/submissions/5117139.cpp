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

template <typename T>
T reversed(T arr) {
    reverse(begin(arr), end(arr));
    return arr;
}

int lis(const vector<int>& arr) {
    //    int n = sz(arr), f[n + 1];
    //    memset(f, 0x3f, sizeof(f));
    //    f[0] = 0;
    //    for (auto& a : arr) {
    //        *lower_bound(f, f + n + 1, a) = a;
    //    }
    //    for (int i = n; i >= 0; i--) {
    //        if (f[i] < 1e9) {
    //            return i;
    //        }
    //    }
    //    assert(false);
    if (!sz(arr)) {
        return 0;
    }
    int n = sz(arr), ans = 0;
    for (int i = 0; i < (1 << n); i++) {
        int prv = -1, cans = 0;
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                if (prv >= arr[j]) {
                    goto loop;
                }
                prv = arr[j];
                cans++;
            }
        }
        if (on(i, n - 1)) {
            ans = max(ans, cans);
        }
    loop:;
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a -= 129;
    }
    int ans = 0;
    for (int i = 0; i < n; i++) {
        ans =
            max(ans, lis(vector<int>(arr, arr + i + 1)) +
                         lis(reversed(vector<int>(arr + i, arr + n))) - 1);
    }
    cout << n - ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}