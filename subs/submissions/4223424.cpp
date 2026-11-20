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
	string a, b;
	cin >> a >> b;
	long acnt[26] {}, bcnt[26] {};
	for (auto& c : a) {
		acnt[c - 'A']++;
	}
	for (auto& c : b) {
		bcnt[c - 'A']++;
	}
	long ans = 0;
	for (int k = 0; k < 26; k++) {
		ans += acnt[k] * bcnt[k];
	}
	for (int i = 0; i < 26; i++) {
		for (int j = 0; j < 26; j++) {
			if (bcnt[i]) {
				bcnt[i]--;
				bcnt[j]++;
				long cans = 0;
				for (int k = 0; k < 26; k++) {
					cans += acnt[k] * bcnt[k];
				}
				bcnt[i]++;
				bcnt[j]--;
				ans = max(ans, cans);
			}
		}
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