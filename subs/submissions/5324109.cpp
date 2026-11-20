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

constexpr int maxn = 1e5 + 5;

double arr[2][maxn];

void solve() {
    int n;
    cin >> n;
    for (int i = 0; i < n; i++) {
        cin >> arr[0][i] >> arr[1][i];
    }
    for (auto& a : arr) {
        sort(a, a + n, greater<>());
    }
    auto check = [&](double x) -> bool {
        int ptr[2] {};
        double sm[2] {};
        while (sm[0] < x || sm[1] < x) {
            for (int i = 0; i < 2; i++) {
                if (sm[i] < x) {
                    if (ptr[i] == n) {
                        return false;
                    }
                    sm[i] += arr[i][ptr[i]++];
                    sm[0]--;
                    sm[1]--;
                }
            }
        }
        return true;
    };
    double l = 0, r = 1e18;
    for (int i = 0; i < 200; i++) {
        double mid = (l + r) / 2;
        if (check(mid)) {
            l = mid;
        } else {
            r = mid;
        }
    }
    cout << l << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(4);
    solve();
}