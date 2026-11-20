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

const int MAXN = 505;

bitset<MAXN> dp[2][MAXN], arr[MAXN];

void solve() {
	int n, m, slen;
	cin >> n >> m >> slen;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for (int j = 0; j < m; j++) {
			arr[i].set(j, s[j] == '.');
		}
	}
	string s;
	cin >> s;
	int pind = 0, cind = 1;
	for (int i = 0; i < n; i++) {
		dp[cind][i].set();
		dp[cind][i] &= arr[i];
	}
	for (auto& c : s) {
		swap(pind, cind);
		for (auto& a : dp[cind]) {
			a.reset();
		}
		auto up = [&]() -> void {
			for (int i = 0; i < n - 1; i++) {
				dp[cind][i] |= dp[pind][i + 1];
			}
		};
		auto down = [&]() -> void {
			for (int i = 1; i < n; i++) {
				dp[cind][i] |= dp[pind][i - 1];
			}
		};
		auto left = [&]() -> void {
			for (int i = 0; i < n; i++) {
				dp[cind][i] |= dp[pind][i] >> 1;
			}
		};
		auto right = [&]() -> void {
			for (int i = 0; i < n; i++) {
				dp[cind][i] |= dp[pind][i] << 1;
			}
		};
		if (c == 'N') {
			up();
		} else if (c == 'S') {
			down();
		} else if (c == 'E') {
			right();
		} else if (c == 'W') {
			left();
		} else {
			assert(c == '?');
			up();
			down();
			left();
			right();
		}
		for (int i = 0; i < n; i++) {
			dp[cind][i] &= arr[i];
		}
	}
	int ans = 0;
	for (auto& a : dp[cind]) {
		ans += int(a.count());
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}