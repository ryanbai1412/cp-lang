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

const int maxn = 3e5, bsize = 400, bcnt = maxn / bsize + 1;

struct Block {
	int suf_min = 1e9, block_min = 1e9, local_min[bsize] {}, each_end[bsize] {},
		each_len[bsize] {};

	Block() {
		fill(begin(local_min), end(local_min), 1e9);
		iota(begin(each_end), end(each_end), 0);
	}

	int query_min(int ind) { return min(suf_min, local_min[ind]); }

	void insert(int l, int r) {
		block_min = min(block_min, r);
		local_min[l] = min(local_min[l], r);
		for (int i = bsize - 2; i >= 0; i--) {
			local_min[i] = min(local_min[i], local_min[i + 1]);
			if (local_min[i] < bsize) {
				each_end[i] = each_end[local_min[i]];
				each_len[i] = each_len[local_min[i]] + 1;
			}
		}
	}
} blocks[bcnt];

void solve() {
	int n, q;
	cin >> q >> n;
	while (q--) {
		int t, l, r;
		cin >> t >> l >> r;
		l--;
		if (t == 1) {
			int cblock = l / bsize, cadd = cblock * bsize;
			blocks[cblock].insert(l - cadd, r - cadd);
			for (int i = bcnt - 2; i >= 0; i--) {
				blocks[i].suf_min =
					min({blocks[i].suf_min, blocks[i + 1].block_min + bsize,
						 blocks[i + 1].suf_min + bsize});
				if (i <= 2) {
					dbg(i, blocks[i].suf_min);
				}
			}
		} else {
			int ans = 0;
			int bend = r / bsize;
			while (l / bsize < bend) {
				int cblock = l / bsize, cl = l % bsize, cadd = cblock * bsize;
				ans += blocks[cblock].each_len[cl];
				l = blocks[cblock].each_end[cl];
				l = cadd + blocks[cblock].query_min(l);
				if (l <= r) {
					ans++;
				}
			}
			int cblock = bend, cadd = cblock * bsize;
			while (l < r) {
				l = cadd + blocks[cblock].local_min[l - cadd];
				if (l <= r) {
					ans++;
				}
			}
			cout << ans << endl;
		}
	}
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