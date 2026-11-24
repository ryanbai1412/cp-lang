// corpus/submissions/4376742.cpp with dp static and fixed-size: its stack VLA of bitsets crashes under clang.
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

const int maxn = 255, maxb = 675 * 130;

using B = bitset<maxb * 2>;

void solve() {
	int n;
	cin >> n;
	string s;
	cin >> s;
	int arr[n];
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] - 'A' + 1;
	}
	int cind = 0, pind = 1;
	static B dp[2][maxn + 1];
	dp[cind][0].set(maxb);
	for (int i = n - 1; i >= 0; i--) {
		swap(cind, pind);
		for (int j = 0; j < max(i + 1, n - i); j++) {
			auto &ans = dp[cind][j];
			ans = dp[pind][j];
			ans |= dp[pind][j + 1] << (arr[i] * arr[i]);
			if (j) {
				ans |= dp[pind][j - 1] >> (arr[i] * arr[i]);
			}
		}
		for (int j = max(i + 1, n - i); j <= n; j++) {
			dp[cind][j].reset();
		}
	}
	auto& ans = dp[cind][0];
	cout << ans.count() << endl;
	for (int i = 0; i < sz(ans); i++) {
		if (ans[i]) {
			cout << i - maxb << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}