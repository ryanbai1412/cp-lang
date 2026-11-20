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

const long mod = 1e9 + 7;

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

struct DS {
	int cnt = 0;
	vector<int> v;

	DS(int n) : v(n + 1) {}

	void update(int l, int r) {
		cnt++;
		v[l]++;
		v[r + 1]--;
	}

	vector<bool> compute() {
		vector<bool> ans(sz(v));
		int cur = 0;
		for (int i = 0; i < sz(v); i++) {
			ans[i] = (cur += v[i]) == cnt;
		}
		return ans;
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n][2];
	for (int i = 0; i < 2; i++) {
		for (int j = 0; j < n; j++) {
			cin >> arr[j][i];
		}
	}
	DS ds[2] {DS(m * 2 + 1), DS(m * 2 + 1)};
	int cnt[2] {};
	for (int i = 0; i < n; i++) {
		if (!arr[i][0] && !arr[i][1]) {
			cnt[i & 1]++;
		} else if (!arr[i][0] || !arr[i][1]) {
			int x = arr[i][0] + arr[i][1];
			ds[i & 1].update(x + 1, x + m);
		} else {
			int x = arr[i][0] + arr[i][1];
			ds[i & 1].update(x, x);
		}
	}
	vector<bool> poss[2] = {ds[0].compute(), ds[1].compute()};
	auto ways = [&](int x) -> int {
		return max(0, min(x - 2, 2 * m - x) + 1);
	};
	auto solve = [&](int ind, int x) -> long {
		if (!poss[ind][x]) {
			return 0;
		}
		return bpow(ways(x), cnt[ind]);
	};
	long ans[2] {};
	for (int i = 0; i <= 2 * m; i++) {
		ans[0] += solve(0, i);
		ans[1] += solve(1, i);
	}
	ans[0] %= mod;
	ans[1] %= mod;
	dbg(ans[0], ans[1]);
	cout << (ans[0] * ans[1]) % mod << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}