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
    long kv;
    cin >> n >> kv;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    vector<long> carr;
    for (auto& a : arr) {
        long csm = accumulate(carr.begin(), carr.begin() + a - 1, long(0));
        long cans = max(long(-1), kv - (csm + a - 1));
        if (cans >= 0) {
            int del = sz(carr) - (a - 1);
            carr.erase(carr.begin() + a - 1, carr.end());
            for (long& b : carr) {
                b++;
            }
            carr.insert(carr.begin(), del, 0);
        }
        carr.insert(upper_bound(begin(carr), end(carr), cans), cans);
        cout << cans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}