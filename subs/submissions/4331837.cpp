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

void solve() {
	int n, m;
	cin >> n >> m;
	string s;
	cin >> s;
	pair<char, string> arr[n];
	for (auto& [c, x] : arr) {
		cin >> c >> x;
	}
	sort(arr, arr + n, [&](const auto &a, const auto &b) -> bool {
		int xa = -sz(a.second), xb = -sz(b.second);
		return tie(xa, a.first) < tie(xb, b.first);
	});
	bool dp[m + 1] {};
	pair<char, int> nxt[m + 1];
	dp[m] = true;
	for (int i = m - 1; i >= 0; i--) {
		for (auto& [c, x] : arr) {
			if (s.substr(i, sz(x)) == x) {
				if (dp[i + sz(x)]) {
					dp[i] = true;
					nxt[i] = {c, i + sz(x)};
					goto loop;
				}
			}
		}
		loop:;
	}
	string ans;
	for (int i = 0; i < m; ) {
		auto [c, x] = nxt[i];
		ans += c;
		i = x;
	}
	cout << ans << endl;
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