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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const long mod1 = 1e9 + 7, mod2 = 998244353;

template<long mod>
long ghash(const vector<int> &arr) {
	long ans = 0;
	for (auto& a : arr) {
		ans = (ans * 31 + a) % mod;
	}
	return ans;
}

long chash(const vector<int> &arr) {
	return ghash<mod1>(arr) * mod2 + ghash<mod2>(arr);
}

long compute(const vector<int>& arr) {
	long sum = accumulate(begin(arr), end(arr), long(0));
	int n = sz(arr);
	long ans = sum * sum;
	for (auto& a : arr) {
		ans += n * a * a - 2 * a * sum;
	}
	return ans;
}

template<bool b>
void solve(const vector<int> &arr) {
	long target = 0;
	int n = sz(arr);
	if (b) {
		target = n / 2 * *max_element(begin(arr), end(arr));
	}
	vector<vector<int>> q;
	set<long> vis;
	long ans = 1e18;
	auto push = [&](const vector<int>& x) -> void {
		long v = chash(x);
		if (vis.insert(v).second) {
			long cval = compute(x);
			if (!b || cval < ans + target) {
				ans = min(ans, cval);
				q.push_back(x);
			}
		}
	};
	push(arr);
	while (sz(q)) {
		auto cur = q.back();
		ans = min(ans, compute(cur));
		q.pop_back();
		for (int i = 1; i < n - 1; i++) {
			int x = cur[i];
			cur[i] = cur[i - 1] + cur[i + 1] - cur[i];
			push(cur);
			cur[i] = x;
		}
	}
	cout << ans << endl;
}

void solve() {
	int n;
	cin >> n;
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
	}
	if (n <= 20) {
		solve<false>(arr);
	} else {
		solve<true>(arr);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}