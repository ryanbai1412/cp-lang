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
#define sz(x) int((x).size())

const int maxn = 3e3 + 5;

int n, k;
long v[maxn];
map<int, long> memo;

long dp(int x) {
	auto it = memo.find(x);
	if (it != memo.end()) {
		return it->second;
	}
	long ans = 1e18;
	if (x < k) {
		ans = v[x];
	}
	if (x <= 2 * maxn + 100) {
		for (int i = 1; i <= k && i < x; i++) {
			ans = min(ans, dp(i) + dp(x - i));
		}
	} else {
		int l = x / 2 - k / 2 - 3, r = x / 2 + k / 2 + 3;
		for (int i = l; i <= r; i++) {
			ans = min(ans, dp(i) + dp(x - i));
		}
	}
	return memo[x] = ans;
}

void solve() {
	cin >> n >> k;
	long arr[k + 1];
	arr[0] = 0;
	for (int i = 1; i <= k; i++) {
		cin >> arr[i];
	}
	for (int i = 1; i + 1 <= k; i++) {
		v[i] = arr[i + 1] + (i - 1) * arr[1];
	}
	cout << dp(n - 2) + 2 * arr[1] << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}