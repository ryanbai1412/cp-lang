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

map<int, long> memo;

int cdiv(int a, int b) {
	return (a + b - 1) / b;
}

long dp(int x) {
	if (x == 1) {
		return 1;
	}
	auto [it, inserted] = memo.emplace(x, 0);
	if (!inserted) {
		return it->second;
	}
	int bord = floor(sqrt(x));
	long ans = 0;
	for (int i = 1; i <= bord; i++) {
		int r = x / i;
		int l = x / (i + 1) + 1;
		ans += (r - l + 1) * dp(i);
	}
	for (int i = 2; i <= bord + 1; i++) {
		int j = x / i;
		if (bord < j && j <= x) {
			dbg(x, j);
			ans += dp(j);
		}
	}
	it->second = ans;
	return ans;
}

void solve() {
	int n;
	cin >> n;
	cout << dp(n) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}