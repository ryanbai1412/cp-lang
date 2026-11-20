#pragma GCC target("avx2")

#include "bits/extc++.h"

using namespace std;

template <class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template <class T, class U = less<T>>
using rt = __gnu_pbds::tree<T,
							__gnu_pbds::null_type,
							U,
							__gnu_pbds::rb_tree_tag,
							__gnu_pbds::tree_order_statistics_node_update>;

template <class T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <class T, class... U>
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

// imagine a language where int = long
#define long int64_t

// typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const long mod = 998244353;

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

long inv(long x) {
	return bpow(x, mod - 2);
}

using mat = vector<vector<long>>;

mat mul(const mat& a, const mat& b) {
	int n = sz(a), m = sz(b), x = sz(b[0]);
	mat ans(n, vector<long>(x));
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			for (int k = 0; k < x; k++) {
				ans[i][k] = (ans[i][k] + a[i][j] * b[j][k]) % mod;
			}
		}
	}
	return ans;
}

long deter(mat arr) {
	int n = sz(arr), mul = 1;
	for (int _ = 0; _ < n; _++) {
		for (int i = 0; i < n; i++) {
			for (int j = i; j < n; j++) {
				if (arr[j][i]) {
					swap(arr[i], arr[j]);
					if (i != j) {
						mul *= -1;
					}
					goto loop;
				}
			}
			for (int j = 0; j < i; j++) {
				if (arr[j][i]) {
					swap(arr[i], arr[j]);
					if (i != j) {
						mul *= -1;
					}
					goto loop;
				}
			}
			return 0;
		loop:;
			long cur = arr[i][i], icur = inv(cur);
			for (auto& a : arr[i]) {
				a = (a * icur) % mod;
			}
			for (int j = i + 1; j < n; j++) {
				long cmul = arr[j][i];
				for (int k = 0; k < n; k++) {
					arr[j][k] = (arr[j][k] - arr[i][k] * cmul) % mod;
					if (arr[j][k] < 0) {
						arr[j][k] += mod;
					}
				}
			}
			for (auto& a : arr[i]) {
				a = (a * cur) % mod;
			}
		}
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < i; j++) {
				if (arr[i][j]) {
					goto loop1;
				}
			}
		}
		break;
	loop1:;
	}
	long ans = mul;
	for (int i = 0; i < n; i++) {
		ans = (ans * arr[i][i]) % mod;
	}
	if (ans < 0) {
		ans += mod;
	}
	return ans;
}

void solve() {
	int k;
	cin >> k;
	int levels[k];
	for (auto& a : levels) {
		cin >> a;
	}
	int ms[k - 1];
	for (auto& a : ms) {
		cin >> a;
	}
	mat cur{};
	for (int i = 0; i < k - 1; i++) {
		int m = ms[i];
		mat arr(levels[i], vector<long>(levels[i + 1]));
		for (int i = 0; i < m; i++) {
			int u, v;
			cin >> u >> v;
			u--;
			v--;
			arr[u][v] = 1;
		}
		if (sz(cur)) {
			cur = mul(cur, arr);
		} else {
			cur = arr;
		}
	}
	cout << deter(cur) << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}