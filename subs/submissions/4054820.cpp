#ifndef LOCAL
#pragma GCC optimize("O3")
#endif

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

namespace io {
char buf[1 << 22];

void init() {
	fread_unlocked(buf, 1, 1 << 22, stdin);
}

void read(int& x) {
	static int ind = 0;
	char c;
	while ((c = buf[ind++]) < '0' || c > '9')
		;
	x = c - '0';
	while ((c = buf[ind++]) >= '0' && c <= '9')
		x = x * 10 + c - '0';
}
}

using io::read;

struct custom_hash {
	static uint64_t splitmix64(uint64_t x) {
		// http://xorshift.di.unimi.it/splitmix64.c
		x += 0x9e3779b97f4a7c15;
		x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9;
		x = (x ^ (x >> 27)) * 0x94d049bb133111eb;
		return x ^ (x >> 31);
	}
	
	size_t operator()(uint64_t x) const {
		static const uint64_t FIXED_RANDOM = chrono::steady_clock::now().time_since_epoch().count();
		return splitmix64(x + FIXED_RANDOM);
	}
};

const int maxn = 1e5 + 5, bsize = 316, bcnt = maxn / bsize + 1;

struct Query {
	int qind, l, r;
};

int cnt[maxn];
pair<int, int> opt {0, 0};
vector<Query> queries[bcnt][bcnt];

void add(int x) {
	opt = max(opt, pair<int, int> {++cnt[x], x});
}

void rm(int x) {
	cnt[x]--;
}

int answer() {
	return opt.second;
}

void solve() {
	int n, q;
	read(n);
	read(q);
	int arr[n];
	for (auto& a : arr) {
		read(a);
		a--;
	}
	int ans[q];
	__gnu_pbds::gp_hash_table<long, int, custom_hash> cache;
	for (int i = 0; i < q; i++) {
		int l, r;
		read(l);
		read(r);
		l--;
		r--;
		int &cur = cache[l * n + r, i];
		if (!cur) {
			cur = i + 1;
			if ((r - l + 1) <= bsize) {
				for (int k = l; k <= r; k++) {
					add(arr[k]);
				}
				ans[i] = answer();
				opt = {0, 0};
				for (int k = l; k <= r; k++) {
					rm(arr[k]);
				}
			} else {
				int lb = (l + bsize - 1) / bsize;
				int rb = (r + 1) / bsize;
				queries[lb][rb].push_back({i, l, r});
			}
		} else {
			ans[i] = -(cur - 1);
		}
	}
	for (int i = 0; i < bcnt; i++) {
		int cl = i * bsize;
		for (int j = i; j < bcnt; j++) {
			int cr = j * bsize;
			for (auto& [qind, l, r] : queries[i][j]) {
				auto cur = opt;
				for (int k = l; k < cl; k++) {
					add(arr[k]);
				}
				for (int k = cr; k <= r; k++) {
					add(arr[k]);
				}
				ans[qind] = answer();
				opt = cur;
				for (int k = l; k < cl; k++) {
					rm(arr[k]);
				}
				for (int k = cr; k <= r; k++) {
					rm(arr[k]);
				}
			}
			for (int k = cr; k < cr + bsize && k < n; k++) {
				add(arr[k]);
			}
		}
		memset(cnt, 0, sizeof(cnt));
		opt = {0, 0};
	}
	for (auto& a : ans) {
		if (a < 0) {
			a = ans[-a];
		}
		cout << a + 1 << endl;
	}
}

int main() {
	io::init();
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