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

const int maxn = 2187;

int cnt[maxn];

void solve() {
	int n, k;
	cin >> n >> k;
	int arr[n] {};
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for (auto& a : s) {
			arr[i] = arr[i] * 3 + (a - '1');
		}
		cnt[arr[i]]++;
	}
	int ans = 0;
	for (int i = 0; i < n; i++) {
		cnt[arr[i]]--;
		for (int j = 0; j < i; j++) {
			cnt[arr[j]]--;
			int cur = 0, pow = 1, x = arr[i], y = arr[j];
			for (int a = 0; a < k; a++) {
				int add;
				if (x % 3 == y % 3) {
					add = x % 3;
				} else {
					add = 0 ^ 1 ^ 2 ^ (x % 3) ^ (y % 3);
				}
				x /= 3;
				y /= 3;
				cur += pow * add;
				pow *= 3;
			}
			ans += cnt[cur];
			cnt[arr[j]]++;
		}
		cnt[arr[i]]++;
	}
	cout << ans / 3 << endl;
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