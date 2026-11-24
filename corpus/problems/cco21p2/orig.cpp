// Reference for CCO '21 P2 at the statement's bounds: the AC submission
// corpus/submissions/4543410.cpp with its layer expansion rewritten. Every change:
// - A layer is expanded per digit with word-parallel bitset shifts instead of one hash
//   insertion per (number, digit) pair. The original costs |layer| * (D / K) hash
//   operations per layer: with K = 2 and all 5001 digits in -2500..=2500 (valid: K >= 2,
//   D <= 5001, |a_i| <= M <= 2500) that is about 6e8 per query, 6-7 s for Q = 5 against
//   the 1.5 s limit. The bitset form is D shifts of a 2^13-bit window per layer.
// - `from` (the digit that reached each number, kept across layers) is unchanged in role;
//   a number takes the first digit that reaches it, so the digits printed can differ from
//   the original's, and the checker accepts any valid spelling.
// Unchanged: the breadth-first layers from x, the 100-layer cap, the stop once 0 has been
// reached, the reconstruction, and the input and output formats.
#include "bits/extc++.h"

using namespace std;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

struct chash {
	uint64_t operator()(uint64_t x) const {
		x ^= x << 17;
		x ^= x >> 13;
		x ^= x << 19;
		x ^= x >> 16;
		return x;
	}
};

constexpr int W = 1 << 13;
using Bits = bitset<W>;

void solve() {
	int base, queries, n, m;
	cin >> base >> queries >> n >> m;
	auto md = [&](long x) -> int {
		x %= base;
		if (x < 0) {
			x += base;
		}
		return int(x);
	};
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
	}
	vector<vector<int>> vals(base);
	for (auto& a : arr) {
		vals[md(a)].push_back(a);
	}
	// (b - r) / K for each digit b of residue r: exact, since b = r (mod K).
	auto quot = [&](long a, int r) { return (a - r) / base; };
	while (queries--) {
		long x;
		cin >> x;
		__gnu_pbds::gp_hash_table<long, int, chash> from;
		vector<long> q = {x};
		for (int i = 0; i < 100; i++) {
			// Group the layer by residue r; a number a of residue r reaches quot(a, r) - quot(b, r).
			map<int, vector<long>> byres;
			for (auto& a : q) {
				int r = md(a);
				if (!vals[r].empty()) {
					byres[r].push_back(quot(a, r));
				}
			}
			long lo = LLONG_MAX, hi = LLONG_MIN;
			for (auto& [r, as] : byres) {
				auto [amin, amax] = minmax_element(as.begin(), as.end());
				for (int b : vals[r]) {
					lo = min(lo, *amin - quot(b, r));
					hi = max(hi, *amax - quot(b, r));
				}
			}
			vector<long> nq;
			if (!byres.empty()) {
				assert(hi - lo < W);
				Bits seen;
				for (auto& [r, as] : byres) {
					long amin = *min_element(as.begin(), as.end());
					Bits s;
					for (long a : as) {
						s[a - amin] = 1;
					}
					for (int b : vals[r]) {
						Bits fresh = (s << (amin - quot(b, r) - lo)) & ~seen;
						seen |= fresh;
						for (size_t j = fresh._Find_first(); j < W; j = fresh._Find_next(j)) {
							long nxt = lo + long(j);
							if (from.insert({nxt, b}).second) {
								nq.push_back(nxt);
							}
						}
					}
				}
			}
			if (from.find(0) != from.end()) {
				long u = 0;
				vector<int> ans;
				do {
					int cx = from[u];
					ans.push_back(cx);
					u = u * base + cx;
				} while (u != x);
				for (int j = 0; j < sz(ans); j++) {
					cout << ans[j] << " \n"[j == sz(ans) - 1];
				}
				goto loop;
			}
			swap(q, nq);
		}
		cout << "IMPOSSIBLE" << endl;
	loop:;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}
