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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using ll = long long;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

long k;
map<long, long> memo;
vector<long> arr, psum;

long dp(long x) {
    if (!x) {
        return 1;
    }
    auto [it, inserted] = memo.emplace(x, 0);
    if (!inserted) {
        return it->second;
    }
    int bit = __lg(x);
    long ans = dp((long(1) << bit) - 1);
    if ((psum[bit] >> bit) >= k) {
        ans += dp(min(x - (long(1) << bit), (psum[bit] - (k << bit)) / k));
    }
    return it->second = ans;
}

ll count_tastiness(ll _k, vector<ll> _arr) {
    k = _k;
    arr.clear();
    arr.resize(64);
    copy(begin(_arr), end(_arr), arr.begin());
    psum.resize(64);
    long cpsum = 0;
    for (int i = 0; i < 64; i++) {
        psum[i] = cpsum += arr[i] << i;
    }
    memo.clear();
    dbg(dp(0), dp(1), dp(2), dp(3), dp(4), dp(5), dp(6));
    return dp(long(1) << 62);
}
// The original is the function the IOI grader calls. This `main` stands in for the
// grader: q, then per call `k x` and a[0..k).
int main() {
	int q;
	if (scanf("%d", &q) != 1) return 1;
	while (q--) {
		int k;
		ll x;
		cin >> k >> x;
		vector<ll> a(k);
		for (auto& v : a) cin >> v;
		cout << count_tastiness(x, a) << "\n";
	}
}
