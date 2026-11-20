#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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

const int maxn = 3e3 + 5, maxs = 2 * maxn;

namespace ht {

const int bsize = 10 << 20, hsize = 1 << 23, hmask = hsize - 1;

int keys[hsize];
long bcache[bsize], values[hsize];

long& lookup(int x) {
	if (x < bsize) {
		return bcache[x];
	}
	int key = x;
	key ^= key << 5;
	key ^= key >> 16;
	key &= hmask;
	for (int i = key;; i++, i &= hmask) {
		if (keys[i] == x) {
			return values[i];
		} else if (!values[i]) {
			keys[i] = x;
			return values[i];
		}
	}
}

}  // namespace ht

int n, k;
long v[maxn], sm[maxs];

long dp(int x) {
	if (x < maxs) {
		return sm[x];
	}
	long& ans = ht::lookup(x);
	if (ans) {
		return ans;
	}
	ans = 1e18;
	int l = x / 2 - k / 2 - 1, r = l + k;
	for (int i = l; i <= r; i++) {
		ans = min(ans, dp(i) + dp(x - i));
	}
	return ans;
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
	for (int i = 1; i < maxs; i++) {
		sm[i] = 1e18;
		for (int j = 1; j <= i && j < k; j++) {
			sm[i] = min(sm[i], sm[i - j] + v[j]);
		}
	}
	cout << dp(n - 2) + 2 * arr[1] << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}