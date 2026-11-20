
namespace __gnu_pbds {
template <typename T>
using ost =
    tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;
}
using __gnu_pbds::ost;
int query_lt(const ost<int>& ost, int x) {
    return int(ost.order_of_key(x));
}
int query(const ost<int>& ost, int ql, int qr) {
    return query_lt(ost, qr + 1) - query_lt(ost, ql);
}
vector<pair<int, int>> merge_dup(const vector<pair<int, int>>& arr) {
    map<int, int> mp;
    for (auto& [ql, qr] : arr) {
        mp[ql]++;
        mp[qr + 1]--;
    }
    vector<pair<int, int>> ans;
    int p_open = -1, c_sum = 0;
    for (auto& [t, dsum] : mp) {
        int p_sum = c_sum;
        c_sum += dsum;
        if ((!!p_sum) != (!!c_sum)) {
            if (c_sum) {
                assert(p_open == -1);
                p_open = t;
            } else {
                assert(p_open != -1);
                ans.emplace_back(p_open, t - 1);
                p_open = -1;
            }
        }
    }
    return ans;
}
vector<array<int, 3>> merge_dup(const vector<array<int, 3>>& arr) {
    map<int, vector<pair<int, int>>> mp;
    for (auto& [x, ql, qr] : arr) {
        mp[x].emplace_back(ql, qr);
    }
    vector<array<int, 3>> ans;
    for (auto& [x, c_qs] : mp) {
        auto n_qs = merge_dup(c_qs);
        for (auto& [ql, qr] : n_qs) {
            ans.push_back({x, ql, qr});
        }
    }
    return ans;
}
long count_inter(const vector<array<int, 3>>& hori, const vector<array<int, 3>>& vert) {
    map<int, vector<array<int, 3>>> mp;
    for (auto& [x, ql, qr] : hori) {
        mp[x].push_back({1, ql, qr});
    }
    for (auto& [y, ql, qr] : vert) {
        mp[ql].push_back({2, y, +1});
        mp[qr + 1].push_back({2, y, -1});
    }
    ost<int> ost;
    long ans = 0;
    for (auto& [_k, evts] : mp) {
        for (auto& [ty, ind, dx] : evts) {
            if (ty == 1) {
                continue;
            }
            if (dx == +1) {
                ost.insert(ind);
            } else {
                ost.erase(ind);
            }
        }
        for (auto& [ty, ql, qr] : evts) {
            if (ty == 2) {
                continue;
            }
            ans += query(ost, ql, qr);
        }
    }
    return ans;
}
void solve() {
    int tc_id, n_rows, n_cols, n;
    cin >> tc_id >> n_rows >> n_cols >> n;
    vector<array<int, 3>> hori, vert, diag;
    for (int i = 0; i < n; i++) {
        int ty, x1, y1, x2, y2;
        cin >> ty >> x1 >> y1 >> x2 >> y2;
        if (ty == 1) {
            if (x1 > x2) {
                swap(x1, x2);
            }
            assert(y1 == y2);
            hori.push_back({y1, x1, x2});
        } else if (ty == 2) {
            if (y1 > y2) {
                swap(y1, y2);
            }
            assert(x1 == x2);
            vert.push_back({x1, y1, y2});
        } else {
            assert(ty == 3);
            assert(y1 - x1 == y2 - x2 && x1 <= x2);
            diag.push_back({y1 - x1, x1, x2});
        }
    }
    hori = merge_dup(hori);
    vert = merge_dup(vert);
    diag = merge_dup(diag);
    long ans = 0;
    for (auto& arr : {hori, vert, diag}) {
        for (auto& [_x, ql, qr] : arr) {
            ans += (qr - ql + 1);
        }
    }
    ans -= count_inter(hori, vert);
    for (auto& [dd, xl, xr] : diag) {
        set<int> bad;
        for (auto& [y, ql, qr] : hori) {
            int x = y - dd;
            if (xl <= x && x <= xr && ql <= x && x <= qr) {
                bad.insert(x);
            }
        }
        for (auto& [x, ql, qr] : vert) {
            int y = x + dd;
            if (xl <= x && x <= xr && ql <= y && y <= qr) {
                bad.insert(x);
            }
        }
        ans -= sz(bad);
    }
    cout << ans << endl;
}
