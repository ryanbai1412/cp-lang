// Matched C++ for `monge_path`: the same online LARSCH (ecnerwala's, from
// proto/ports/ecnerwala/cf2183h_orig.cpp), C++17, on dp[i] = min over j < i of
// dp[j] + (pre[i] - pre[j])^2 + c.
#include <bits/stdc++.h>
typedef long long ll;

template <typename T> struct value_t {
	T v;
	int col;
};

template <typename Get, typename Select, typename T = std::invoke_result_t<Get, int, int>>
class LARSCH {
public:
	int N;
	Get get;
	Select select;
	int L;
	int num_rows;

	std::vector<std::vector<value_t<T>>> stk;
	std::vector<std::pair<value_t<T>, int>> bests;
	LARSCH() {}
	LARSCH(int N_, Get&& get_, Select&& select_) : N(N_), get(std::forward<Get>(get_)), select(std::forward<Select>(select_)) {
		L = N ? 31 - __builtin_clz(N) : 0;
		stk.resize(L);
		bests.resize(L);
		// N >> L == 1, unless N == 0
		for (int i = 0; i < L; i++) {
			stk[i].reserve(N >> (i+1));
		}
		num_rows = 0;
	}

	value_t<T> push_and_query_next() {
		assert(num_rows < N);
		int inp_row = num_rows++;

		int l = 0;
		value_t<T> nbest;
		while (true) {
			int r = inp_row >> l;
			int col = l == 0 ? inp_row : stk[l-1][r].col;
			if (l == L) {
				// special case: just return the unique element
				assert(r == 0);
				assert(inp_row == 0);
				int row = ((r+1) << l) - 1;
				if (l == 0) nbest = {get(row, col), col};
				else nbest = {std::move(stk[l-1][r].v), stk[l-1][r].col};
				l--;
				break;
			}
			assert(l < L);

			if (r & 1) {
				int row = ((r+1) << l) - 1;
				value_t<T> prv_col_top;
				if (l == 0) prv_col_top = {get(row, col), col};
				else prv_col_top = {std::move(stk[l-1][r].v), stk[l-1][r].col};

				assert(bests[l].second <= r);

				// just check this guy at this row, and then push it into the next layer, but don't query yet
				if (bests[l].second == r || select(row, bests[l].first, prv_col_top)) {
					// prv_col_top is better here
					bests[l].first = std::move(prv_col_top);
					bests[l].second = r;
				}
			}
			if (bests[l].second == r) {
				// We've committed to the new column being the best, so let's prune the stack and propagate to ensure consistency.
				assert(int(stk[l].size()) >= (r+1)/2);
				stk[l].resize((r+1)/2);
				// We can just set .second only, since the only time it's read is the r&1 case, which will override bests[l].first
				if (l+1 < L) bests[l+1].second = (r+1)/2;
			}
			std::optional<value_t<T>> to_push;
			while (int(stk[l].size()) > (r+1)/2) {
				int row = (int(stk[l].size()) << (l+1)) - 1;
				value_t<T> nv{get(row, col), col};
				if (select(row, stk[l].back(), nv)) {
					stk[l].pop_back();
					to_push = std::move(nv);
				} else {
					break;
				}
			}
			if (to_push) {
				stk[l].emplace_back(std::move(*to_push));
			} else {
				int row = (int(stk[l].size()+1) << (l+1)) - 1;
				if (row < N) stk[l].push_back({get(row, col), col});
			}
			if (r & 1) {
				// go return
				nbest = std::move(bests[l].first);
				l--;
				break;
			} else if (((r+2) << l) - 1 >= N) {
				// go return
				nbest.col = col;
				break;
			} else {
				l++;
				continue;
			}
			assert(false);
		}
		for (; l >= 0; l--) {
			int r = inp_row >> l;
			assert(!(r & 1));
			int row = ((r+1) << l) - 1;
			bests[l].first = std::move(nbest);
			bool did_set = false;
			while (true) {
				int idx = bests[l].second;
				assert(idx <= r);
				int col = (l == 0 ? idx : stk[l-1][idx].col);
				assert(col <= bests[l].first.col);
				value_t<T> cnd;
				if (l > 0 && idx == r) cnd = {std::move(stk[l-1][r].v), col};
				else cnd = {get(row, col), col};
				if (!did_set || select(row, nbest, cnd)) {
					did_set = true;
					nbest = std::move(cnd);
				}
				if (col == bests[l].first.col) break;
				bests[l].second++;
			}
		}
		assert(l == -1);
		return nbest;
	}
};

int main() {
	int n; ll c; scanf("%d %lld", &n, &c);
	std::vector<ll> pre(n + 1, 0);
	for (int i = 0; i < n; i++) { ll x; scanf("%lld", &x); pre[i + 1] = pre[i] + x; }
	std::vector<ll> dp(n + 1, 0);
	std::vector<int> prv(n + 1, 0);
	auto get = [&](int row, int col) -> ll { ll d = pre[row + 1] - pre[col]; return dp[col] + d * d + c; };
	auto select = [&](int, const value_t<ll>& a, const value_t<ll>& b) { return b.v <= a.v; };
	LARSCH<decltype(get)&, decltype(select)&, ll> larsch(n, get, select);
	for (int i = 1; i <= n; i++) { auto b = larsch.push_and_query_next(); dp[i] = b.v; prv[i] = b.col; }
	ll hops = 0;
	for (int i = n; i > 0; i = prv[i]) hops++;
	printf("%lld %lld\n", dp[n], hops);
}
