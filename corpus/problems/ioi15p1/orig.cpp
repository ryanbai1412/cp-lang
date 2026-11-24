#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

ll delivery(int n, int kv, int m, int arr[]) {
    vector<long> dp1(n), dp2(n);
    auto g_dp1 = [&](int ind) -> long {
        return ind == -1 ? 0 : dp1[ind];
    };
    auto g_dp2 = [&](int ind) -> long {
        return ind == n ? 0 : dp2[ind];
    };

    for (int i = 0; i < n; i++) {
        dp1[i] = 2 * arr[i] + g_dp1(max(-1, i - kv));
    }
    for (int i = n - 1; i >= 0; i--) {
        dp2[i] = 2 * (m - arr[i]) + g_dp2(min(n, i + kv));
    }

    long ans = 1e18;

    auto dp3 = [&](int ind) -> long {
        if (ind == n) {
            return 0;
        }

        return m + g_dp2(min(n, ind + kv));
    };

    for (int i = 0; i <= n; i++) {
        ans = min(ans, g_dp1(i - 1) + min(dp3(i), g_dp2(i)));
    }

    return ans;
}
// The original is the function the IOI grader calls. This `main` stands in for the
// grader: n k l, then the n positions in order.
int main() {
	int n, k, l;
	if (scanf("%d %d %d", &n, &k, &l) != 3) return 1;
	vector<int> p(n);
	for (auto& v : p) scanf("%d", &v);
	cout << delivery(n, k, l, p.data()) << "\n";
}
