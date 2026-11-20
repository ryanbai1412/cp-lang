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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 305;

int n;
ld memo[maxn][maxn][maxn];

ld dp(int a, int b, int c) {
	ld& ans = memo[a][b][c];
	if (ans >= 0) {
		return ans;
	}
	ans = 0;
	ld cn = a + b + c;
	if (a) {
		ans += (a / cn) * ((ld(n) / cn) + dp(a - 1, b, c));
	}
	if (b) {
		ans += (b / cn) * ((ld(n) / cn) + dp(a + 1, b - 1, c));
	}
	if (c) {
		ans += (c / cn) * ((ld(n) / cn) + dp(a, b + 1, c - 1));
	}
	dbg(a, b, c, ans);
	return ans;
}

void solve() {
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int cnt[4] {};
	for (auto& a : arr) {
		cnt[a]++;
	}
	for (auto& a : memo) {
		for (auto& b : a) {
			fill(begin(b), end(b), -1);
		}
	}
	cout << dp(cnt[1], cnt[2], cnt[3]) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	cout << fixed << setprecision(9);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}