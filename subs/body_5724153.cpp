
struct PSA {
    vector<long> psum;
    PSA(const vector<int>& arr) : psum(sz(arr) + 1) {
        for (int i = 0; i < sz(arr); i++) {
            psum[i + 1] = psum[i] + arr[i];
        }
    }
    long query(int l, int r) const {
        return psum[r] - psum[l];
    }
};
ll min_total_length(vector<int> arr_r, vector<int> arr_b) {
    vector<pair<int, bool>> arr;
    for (auto& a : arr_r) {
        arr.emplace_back(a, false);
    }
    for (auto& a : arr_b) {
        arr.emplace_back(a, true);
    }
    sort(begin(arr), end(arr));
    int n = sz(arr);
    long dp[n + 1];
    memset(dp, 0x3f, sizeof(dp));
    dp[n] = 0;
    deque<int> inds[2];
    int psum_c[n + 1] {};
    for (int i = 0; i < n; i++) {
        psum_c[i + 1] = psum_c[i];
        if (arr[i].second) {
            psum_c[i + 1]++;
        } else {
            psum_c[i + 1]--;
        }
    }
    int p_occ[n + 1][2];
    p_occ[0][0] = p_occ[0][1] = -1;
    for (int i = 0; i < n; i++) {
        memcpy(p_occ[i + 1], p_occ[i], sizeof(p_occ[i]));
        p_occ[i + 1][arr[i].second] = i;
    }
    PSA psa_w(({
        vector<int> v;
        for (auto& [t, ty] : arr) {
            if (ty) {
                v.push_back(t);
            } else {
                v.push_back(-t);
            }
        }
        v;
    }));
    int last_p[2 * n + 10];
    memset(last_p, -1, sizeof(last_p));
    long dp2[n];
    auto upd_dp2 = [&](int j) -> void {
        // for (; j < n; j++) {
        //     dp[i] = min(dp[i], wsum + min(dp[j], dp[j + 1]));
        //     if (j + 1 < n) {
        //         if (arr[j + 1].second != arr[j].second) {
        //             break;
        //         }
        //         wsum += arr[j + 1].first - last_m;
        //     }
        // }
        dp2[j] = min(dp[j], dp[j + 1]);
        if (j + 1 < n) {
            if (arr[j + 1].second != arr[j].second) {
                return;
            }
            dp2[j] = min(dp2[j], arr[j + 1].first -
                                     arr[p_occ[j][!arr[j].second]].first +
                                     dp2[j + 1]);
        }
    };
    for (int i = n - 1; i >= 0; i--) {
        last_p[psum_c[i + 1] + n] = i;
        if (i + 1 < n) {
            inds[arr[i + 1].second].push_front(i + 1);
            upd_dp2(i + 1);
        }
        auto [t, ty] = arr[i];
        auto &me = inds[ty], &other = inds[!ty];
        auto get_w = [&](int u, int v) -> long {
            return abs(arr[u].first - arr[v].first);
        };
        auto get = [&](const deque<int>& arr, int ind) -> int {
            if (ind >= sz(arr)) {
                return n;
            }
            return arr[ind];
        };
        if (!sz(other)) {
            continue;
        }
        if (get(other, 0) < get(me, 0)) {
            long csum = 0;
            for (int j = 0; j < sz(other) && get(other, j) < get(me, 0); j++) {
                csum += get_w(i, other[j]);
                dp[i] = min(dp[i], csum + min(dp[other[j]], dp[other[j] + 1]));
            }
        } else {
            dp[i] = dp[i + 1] + get_w(i, other[0]);
            {
                int j, cnt = 0;
                {
                    int it = last_p[psum_c[i] + n];
                    if (it == -1) {
                        cnt = 1;
                        assert(cnt);
                    } else {
                        j = it;
                    }
                }
                if (!cnt) {
                    dp[i] = min(dp[i], abs(psa_w.query(i, j + 1)) + dp2[j]);
                    // for (; j < n; j++) {
                    //     dp[i] = min(dp[i], wsum + min(dp[j], dp[j + 1]));
                    //     if (j + 1 < n) {
                    //         if (arr[j + 1].second != arr[j].second) {
                    //             break;
                    //         }
                    //         wsum += arr[j + 1].first - last_m;
                    //     }
                    // }
                }
            }
            // long csum = 0;
            // me.push_front(i);
            // for (int j = 0; j < sz(other) && j < sz(me); j++) {
            //     long xsum = csum;
            //     for (int k = j; k < sz(other); k++) {
            //         xsum += get_w(me[j], other[k]);
            //         dp[i] =
            //             min(dp[i],
            //                 xsum + dp[min(get(other, k + 1), get(me, j +
            //                 1))]);
            //     }
            //     csum += get_w(me[j], other[j]);
            // }
            // me.pop_front();
        }
    }
    return dp[0];
}