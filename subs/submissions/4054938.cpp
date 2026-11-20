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
}  // namespace io

using io::read;

const int maxn = 1e5 + 5, bsize = 400;

struct Query {
	int qind, l, r;

	int blockl() const { return (l + bsize - 1) / bsize; }

	int blockr() const { return r < bsize ? -1 : (r - bsize + 1) / bsize; }

	bool operator<(const Query& query) const {
		int al = blockl(), bl = query.blockl();

		if (al == bl) {
			int ar = blockr(), br = query.blockr();
			if (ar == br) {
				return l > query.l;
			}
			return ar < br;
		}

		return al < bl;
	}
};

int cnt[maxn];
pair<int, int> opt {0, 0};

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
	int arr[n], comp[n];
	for (int i = 0; i < n; i++) {
		read(arr[i]);
		comp[i] = arr[i];
	}
	sort(comp, comp + n);
	auto cend = unique(comp, comp + n);
	for (auto& a : arr) {
		a = lower_bound(comp, cend, a) - comp;
	}
	if (*max_element(arr, arr + n) <= 30) {
		int psum[30][n + 1];
		for (int i = 0; i < 30; i++) {
			psum[i][0] = 0;
			for (int j = 0; j < n; j++) {
				psum[i][j + 1] = psum[i][j] + (arr[j] == i);
			}
		}
		for (int i = 0; i < q; i++) {
			int l, r;
			read(l);
			read(r);
			l--;
			r--;
			pair<int, int> ans {0, 0};
			for (int i = 0; i < 30; i++) {
				ans = max(ans, pair<int, int> {psum[i][r + 1] - psum[i][l], i});
			}
			cout << comp[ans.second] << endl;
		}
		return;
	}
	int ans[q];
	vector<Query> queries;
	queries.reserve(n);
	for (int i = 0; i < q; i++) {
		int l, r;
		read(l);
		read(r);
		l--;
		r--;
		if ((r - l + 1) <= bsize) {
			for (int k = l; k <= r; k++) {
				add(arr[k]);
			}
			ans[i] = answer();
			opt = {0, 0};
			for (int k = l; k <= r; k++) {
				cnt[arr[k]] = 0;
			}
		} else {
			queries.push_back({i, l, r});
		}
	}
	sort(begin(queries), end(queries));
	int pl = -1, pr = -1, addedl = 0;
	pair<int, int> blhist = {0, 0};
	for (auto& q : queries) {
		auto& [i, l, r] = q;
		int cl = q.blockl(), cr = q.blockr();
		if (pl != cl) {
			memset(cnt, 0, sizeof(cnt));
			opt = blhist = {0, 0};
			pl = cl;
			pr = cl - 1;
			addedl = pl * bsize;
		}
		if (pr < cr) {
			opt = blhist;
			for (int i = addedl; i < cl * bsize; i++) {
				rm(arr[i]);
			}
			addedl = cl * bsize;
			for (; pr < cr; pr++) {
				int itl = (pr + 1) * bsize, itr = itl + bsize;
				for (int i = itl; i < itr; i++) {
					add(arr[i]);
				}
			}
			blhist = opt;
		}
		for (; addedl > l; addedl--) {
			add(arr[addedl - 1]);
		}
		auto hist = opt;
		for (int i = (pr + 1) * bsize; i <= r; i++) {
			add(arr[i]);
		}
		ans[i] = answer();
		opt = hist;
		for (int i = (pr + 1) * bsize; i <= r; i++) {
			rm(arr[i]);
		}
	}
	for (auto& a : ans) {
		if (a < 0) {
			a = ans[-a];
		}
		cout << a + 1 << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	io::init();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}