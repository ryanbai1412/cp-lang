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
    int arri[n], arr[n];
    for (auto& a : arri) {
        cin >> a;
    }
    for (auto& a : arr) {
        cin >> a;
    }
    sort(arr, arr + n);
    int l = 0, r = n, ans[n];
    memset(ans, -1, sizeof(ans));
    for (int i = 1; i < n - 1; i++) {
        if (arri[i] < arri[i - 1] && arri[i] < arri[i + 1]) {
            ans[i] = arr[l++];
        } else if (arri[i] > arri[i - 1] && arri[i] > arri[i + 1]) {
            ans[i] = arr[--r];
        }
    }
    if (arri[0] < arri[1]) {
        ans[0] = arr[l++];
    } else {
        ans[0] = arr[--r];
    }
    if (arri[n - 1] < arri[n - 2]) {
        ans[n - 1] = arr[l++];
    } else {
        ans[n - 1] = arr[--r];
    }
    for (int i = 1; i < n - 1; i++) {
        if (ans[i] == -1) {
            if (arri[i - 1] < arri[i]) {
                ans[i] = arr[l++];
            } else {
                ans[i] = arr[--r];
            }
        }
    }
    long xans = 0;
    for (int i = 0; i < n - 1; i++) {
        xans += abs(ans[i] - ans[i + 1]);
    }
    cout << xans << endl;
    for (int i = 0; i < n; i++) {
        cout << ans[i] << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}