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

long sq(long x) {
    return x * x;
}

void solve(const vector<int>& arr) {
    int n = sz(arr), psum[n + 1];
    partial_sum(begin(arr), end(arr), psum + 1);
    psum[0] = 0;
    vector<int> part;
    part.push_back(0);
    for (int i = 1, prv = 0; i <= n; i++) {
        int cl = part.back();
        while (psum[i] - psum[cl] < prv) {
            i++;
        }
        if (psum[n] - psum[i] < psum[i] - psum[cl]) {
            i = n;
        }
        dbg(i);
        part.push_back(i);
        if (sz(part) >= 3) {
            int ql = part[sz(part) - 3], &mid = part[sz(part) - 2],
                qr = part.back();
            while (psum[mid] - psum[ql] <= psum[qr] - psum[mid]) {
                mid++;
            }
            mid--;
        }
        prv = psum[part.back()] - psum[part[sz(part) - 2]];
    }
    long ans = 0;
    for (int i = 0; i < sz(part) - 1; i++) {
        ans += sq(psum[part[i + 1]] - psum[part[i]]);
    }
    cout << ans << endl;
}

void solve() {
    int n, tc_type;
    cin >> n >> tc_type;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    solve(arr);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}