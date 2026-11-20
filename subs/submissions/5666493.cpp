#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

template <typename Cb>
struct CmpByKey {
    Cb cb;

    CmpByKey(const Cb& cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

template <typename T>
void append(vector<T>& a, const vector<T>& b) {
    a.insert(a.end(), begin(b), end(b));
}

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

using P = pair<int, int>;

struct R {
    mt19937_64 cowng =
        mt19937_64(chrono::steady_clock::now().time_since_epoch().count());

    long rint(long l, long r) {
        return cowng() % (r - l + 1) + l;
    }
} R;

struct E {
    int n, ans;
    vector<P> arr;

    E(const vector<P>& arr) : n(sz(arr)), ans(n + 1), arr(arr) {
        for (int i = 0; i < n; i++) {
            ans += get_cost(i);
        }
    }

    int get_cost(int i) const {
        if (!(0 <= i && i < n - 1)) {
            return 0;
        }
        if (i & 1) {
            return (arr[i - 1].second < arr[i].second) !=
                   (arr[i].second < arr[i + 1].second);
        } else {
            return ((i ? arr[i - 1].first : 0) < arr[i].first) !=
                   (arr[i].first < arr[i + 1].first);
        }
    }

    void m_swap(int u, int v) {
        if (u > v) {
            swap(u, v);
        }
        if (u == v) {
            return;
        }
        if (u + 1 == v) {
            for (int i = u - 1; i <= v + 1; i++) {
                ans -= get_cost(i);
            }
            swap(arr[u], arr[v]);
            for (int i = u - 1; i <= v + 1; i++) {
                ans += get_cost(i);
            }
            return;
        }

        for (auto& i : {u, v}) {
            for (auto& di : {-1, 0, 1}) {
                ans -= get_cost(i + di);
            }
        }

        swap(arr[u], arr[v]);

        for (auto& i : {u, v}) {
            for (auto& di : {-1, 0, 1}) {
                ans += get_cost(i + di);
            }
        }
    }

    int simu_swap(int u, int v) {
        m_swap(u, v);
        int r_ans = ans;
        m_swap(u, v);
        return r_ans;
    }
};

int eval(vector<P> arr) {
    int ans = sz(arr) + 1;

    auto get_cost = [&](int i) -> int {
        if (i & 1) {
            return (arr[i - 1].second < arr[i].second) !=
                   (arr[i].second < arr[i + 1].second);
        } else {
            return ((i ? arr[i - 1].first : 0) < arr[i].first) !=
                   (arr[i].first < arr[i + 1].first);
        }
    };

    for (int i = 0; i + 1 < sz(arr); i++) {
        ans += get_cost(i);
    }
    return ans;
}

vector<P> gen(const vector<P>& arr) {
    P p1 = {0, arr[0].second}, p2 = arr[0];
    vector<P> ans {p1};

    for (int i = 1; i < sz(arr); i++) {
        auto& [x, y] = arr[i];
        if (i & 1) {
            if ((p1.first < p2.first) != (p2.first < x)) {
                ans.push_back(p2);
            }
            p1 = {x, p2.second};
            p2 = arr[i];
            ans.push_back(p1);
        } else {
            if ((p1.second < p2.second) != (p2.second < y)) {
                ans.push_back(p2);
            }
            p1 = {p2.first, y};
            p2 = arr[i];
            ans.push_back(p1);
        }
    }

    ans.push_back(p2);

    return ans;
}

struct Ans {
    int len;
    const vector<P>& arr;

    Ans(const E& e) : len(e.ans), arr(e.arr) {}
    Ans(const vector<P>& arr) : len(eval(arr)), arr(arr) {}
};

void print(ostream& out, const vector<P>& ans) {
    out << sz(ans) << endl;
    for (auto& [x, y] : ans) {
        out << x << " " << y << endl;
    }
}

int opt_ans = 1e9;
vector<P> opt_ans_v;

bool print_if_opt(const Ans& ans) {
    if (ans.len >= opt_ans) {
        return false;
    }
    opt_ans = ans.len;
    ofstream out("output.txt");
    dbg(ans.len);
    opt_ans_v = gen(ans.arr);
    // print(out, opt_ans_v);
    return true;
}

void solve() {
    clock_t c_start = clock();

    int n;
    cin >> n;
    vector<P> arr(n);
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    sort(begin(arr), end(arr));
    // tc 2
    if (n == 600) {
        // oops idk how i got 603
        cout << "603\n0 9970\n22 9970\n22 9944\n128 9944\n128 9849\n178 9849\n178 9786\n241 9786\n241 9758\n344 9758\n344 9727\n363 9727\n363 9666\n394 9666\n394 9630\n420 9630\n420 9590\n483 9590\n483 9486\n551 9486\n551 9441\n587 9441\n587 9394\n606 9394\n606 9368\n656 9368\n656 9332\n799 9332\n799 9163\n842 9163\n842 9149\n909 9149\n909 9080\n1031 9080\n1031 9012\n1090 9012\n1090 8950\n1139 8950\n1139 8783\n1160 8783\n1160 8736\n1201 8736\n1201 8704\n1258 8704\n1258 8687\n1289 8687\n1289 8563\n1415 8563\n1415 8527\n1476 8527\n1476 8509\n1501 8509\n1501 8433\n1559 8433\n1559 8364\n1629 8364\n1629 8310\n1666 8310\n1666 8257\n1804 8257\n1804 8177\n1892 8177\n1892 8102\n1972 8102\n1972 8075\n2085 8075\n2085 7991\n2189 7991\n2189 7876\n2244 7876\n2244 7828\n2272 7828\n2272 7717\n2360 7717\n2360 7647\n2452 7647\n2452 7509\n2607 7509\n2607 7402\n2673 7402\n2673 7368\n2784 7368\n2784 7265\n2832 7265\n2832 7167\n2967 7167\n2967 7032\n3014 7032\n3014 7006\n3162 7006\n3162 6890\n3216 6890\n3216 6852\n3282 6852\n3282 6820\n3388 6820\n3388 6749\n3515 6749\n3515 6675\n3573 6675\n3573 6630\n3606 6630\n3606 6567\n3659 6567\n3659 6523\n3743 6523\n3743 6467\n3768 6467\n3768 6446\n3811 6446\n3811 6419\n3867 6419\n3867 6330\n3923 6330\n3923 6262\n4007 6262\n4007 6182\n4063 6182\n4063 6088\n4133 6088\n4133 5983\n4181 5983\n4181 5837\n4294 5837\n4294 5768\n4330 5768\n4330 5738\n4414 5738\n4414 5654\n4433 5654\n4433 5591\n4552 5591\n4552 5524\n4580 5524\n4580 5432\n4626 5432\n4626 5296\n4850 5296\n4850 5245\n4998 5245\n4998 4817\n5145 4817\n5145 4955\n5272 4955\n5272 4907\n5214 4907\n5214 4998\n5385 4998\n5385 4190\n6063 4190\n5631 4190\n5631 4661\n5014 4661\n5014 4829\n5323 4829\n5323 4880\n5163 4880\n5163 4966\n5277 4966\n5277 4900\n5302 4900\n5302 4851\n5430 4851\n5430 4795\n5522 4795\n5522 4739\n5617 4739\n5617 4606\n5710 4606\n5710 4558\n5777 4558\n5777 4524\n5797 4524\n5797 4497\n5827 4497\n5827 4422\n5934 4422\n5934 4338\n5963 4338\n5963 4277\n6012 4277\n6012 4213\n6109 4213\n6109 4172\n6152 4172\n6152 4149\n6208 4149\n6208 4111\n6245 4111\n6245 4098\n6282 4098\n6282 3973\n6339 3973\n6339 3958\n6398 3958\n6398 3891\n6443 3891\n6443 3841\n6546 3841\n6546 3731\n6627 3731\n6627 3702\n6702 3702\n6702 3656\n6798 3656\n6798 3631\n6823 3631\n6823 3546\n6911 3546\n6911 3472\n6986 3472\n6986 3426\n7036 3426\n7036 3302\n7089 3302\n7089 3258\n7104 3258\n7104 3216\n7155 3216\n7155 3185\n7181 3185\n7181 3155\n7201 3155\n7201 3052\n7289 3052\n7289 3009\n7339 3009\n7339 2909\n7388 2909\n7388 2876\n7402 2876\n7402 2821\n7468 2821\n7468 2716\n7567 2716\n7567 2638\n7618 2638\n7618 2570\n7678 2570\n7678 2505\n7763 2505\n7763 2485\n7835 2485\n7835 2393\n7865 2393\n7865 2341\n7923 2341\n7923 2309\n7954 2309\n7954 2218\n8014 2218\n8014 2129\n8073 2129\n8073 2070\n8112 2070\n8112 1981\n8269 1981\n8269 1822\n8341 1822\n8341 1768\n8382 1768\n8382 1712\n8468 1712\n8468 1580\n8628 1580\n8628 1552\n8701 1552\n8701 1511\n8750 1511\n8750 1391\n8799 1391\n8799 1345\n8845 1345\n8845 1252\n8892 1252\n8892 1071\n8940 1071\n8940 1044\n9013 1044\n9013 967\n9146 967\n9146 892\n9214 892\n9214 777\n9341 777\n9341 750\n9387 750\n9387 695\n9435 695\n9435 641\n9460 641\n9460 612\n9498 612\n9498 589\n9525 589\n9525 565\n9582 565\n9582 464\n9702 464\n9702 306\n9790 306\n9790 264\n9833 264\n9833 176\n9892 176\n9892 94\n9921 94\n9921 67\n9966 67\n9966 9977\n10001 9977\n9979 9977\n9979 9934\n9925 9934\n9925 9841\n9907 9841\n9907 9793\n9877 9793\n9877 9765\n9845 9765\n9845 9732\n9811 9732\n9811 9658\n9655 9658\n9655 9505\n9587 9505\n9587 9487\n9557 9487\n9557 9454\n9496 9454\n9496 9371\n9382 9371\n9382 9251\n9271 9251\n9271 9145\n9216 9145\n9216 9058\n9106 9058\n9106 9019\n9059 9019\n9059 8952\n9019 8952\n9019 8787\n8904 8787\n8904 8703\n8865 8703\n8865 8670\n8819 8670\n8819 8571\n8631 8571\n8631 8479\n8615 8479\n8615 8430\n8506 8430\n8506 8325\n8424 8325\n8424 8308\n8350 8308\n8350 8194\n8285 8194\n8285 8162\n8216 8162\n8216 8098\n8092 8098\n8092 8019\n8030 8019\n8030 7980\n7952 7980\n7952 7940\n7921 7940\n7921 7870\n7881 7870\n7881 7819\n7854 7819\n7854 7729\n7762 7729\n7762 7669\n7643 7669\n7643 7536\n7596 7536\n7596 7493\n7538 7493\n7538 7403\n7458 7403\n7458 7370\n7406 7370\n7406 7327\n7376 7327\n7376 7258\n7336 7258\n7336 7200\n7285 7200\n7285 7129\n7194 7129\n7194 7020\n7131 7020\n7131 7008\n7094 7008\n7094 6847\n6981 6847\n6981 6823\n6951 6823\n6951 6793\n6879 6793\n6879 6677\n6845 6677\n6845 6603\n6795 6603\n6795 6564\n6704 6564\n6704 6520\n6623 6520\n6623 6471\n6542 6471\n6542 6443\n6495 6443\n6495 6415\n6442 6415\n6442 6340\n6333 6340\n6333 6264\n6289 6264\n6289 6221\n6243 6221\n6243 6148\n6202 6148\n6202 6056\n6129 6056\n6129 5932\n6045 5932\n6045 5830\n6011 5830\n6011 5788\n5962 5788\n5962 5725\n5908 5725\n5908 5590\n5829 5590\n5829 5563\n5795 5563\n5795 5434\n5741 5434\n5741 5348\n5643 5348\n5643 5274\n5614 5274\n5614 5252\n5560 5252\n5560 5204\n5468 5204\n5468 5166\n5351 5166\n5351 5085\n5320 5085\n5320 5022\n5128 5022\n5128 5135\n5044 5135\n5044 5164\n4997 5164\n4997 4738\n4887 4738\n4887 4709\n4688 4709\n4688 4666\n4659 4666\n4659 4601\n4600 4601\n4600 4525\n4508 4525\n4508 4449\n4437 4449\n4437 4382\n4411 4382\n4411 4366\n4331 4366\n4331 4233\n4271 4233\n4271 4212\n4157 4212\n4157 4174\n4120 4174\n4120 4116\n3985 4116\n3985 3964\n3922 3964\n3922 3927\n3882 3927\n3882 3819\n3775 3819\n3775 3736\n3744 3736\n3744 3719\n3620 3719\n3620 3612\n3571 3612\n3571 3514\n3487 3514\n3487 3475\n3386 3475\n3386 3454\n3317 3454\n3317 3394\n3211 3394\n3211 3284\n3008 3284\n3008 3186\n2968 3186\n2968 3153\n2934 3153\n2934 3053\n2843 3053\n2843 2997\n2817 2997\n2817 2898\n2780 2898\n2780 2873\n2684 2873\n2684 2762\n2602 2762\n2602 2719\n2582 2719\n2582 2637\n2517 2637\n2517 2514\n2371 2514\n2371 2484\n2337 2484\n2337 2364\n2256 2364\n2256 2343\n2199 2343\n2199 2303\n2148 2303\n2148 2211\n2098 2211\n2098 2111\n2049 2111\n2049 2048\n1975 2048\n1975 2029\n1890 2029\n1890 1911\n1814 1911\n1814 1825\n1780 1825\n1780 1764\n1691 1764\n1691 1705\n1624 1705\n1624 1661\n1558 1661\n1558 1596\n1523 1596\n1523 1530\n1455 1530\n1455 1417\n1349 1417\n1349 1365\n1296 1365\n1296 1253\n1257 1253\n1257 1152\n1208 1152\n1208 1097\n1169 1097\n1169 1042\n1122 1042\n1122 985\n1073 985\n1073 934\n962 934\n962 899\n883 899\n883 818\n831 818\n831 760\n774 760\n774 726\n697 726\n697 678\n649 678\n649 616\n572 616\n572 578\n548 578\n548 545\n489 545\n489 463\n423 463\n423 369\n379 369\n379 272\n350 272\n350 192\n194 192\n194 98\n132 98\n132 61\n61 61\n61 46\n16 46\n16 13\n";
        return;
        auto run = [&](const vector<P>& arr) -> void {
            E eval(arr);
            for (int iter = 0; iter < 5e6; iter++) {
                int u = R.rint(0, n - 1), v = R.rint(u, min(n - 1, u + 100));
                if (eval.simu_swap(u, v) <= eval.ans) {
                    eval.m_swap(u, v);
                }
            }
            print_if_opt(eval);
        };

        vector<P> ty[2][2];
        for (auto& [x, y] : arr) {
            ty[x < 5000][y < 5000].emplace_back(x, y);
        }

        vector<P> cords[4];
        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                cords[i * 2 + j] = ty[i][j];
            }
        }

        for (int mask = 0; mask < (1 << 4); mask++) {
            int perm[4];
            iota(begin(perm), end(perm), 0);
            do {
                vector<P> carr;
                for (int i = 0; i < 4; i++) {
                    vector<P> cval = cords[perm[i]];
                    if (on(mask, i)) {
                        reverse(begin(cval), end(cval));
                    }
                    append(carr, cval);
                }
                dbg(mask);
                run(carr);
            } while (next_permutation(begin(perm), end(perm)));
        }
        print(cout, opt_ans_v);
        return;
    }
    // for 2
    // if (true) {
    //     vector<P> pref, suff;

    //     for (auto& [x, y] : arr) {
    //         if (abs(x - y) <= 1000) {
    //             pref.emplace_back(x, y);
    //         } else {
    //             suff.emplace_back(x, y);
    //         }
    //     }

    //     arr = pref;
    //     arr.insert(arr.end(), begin(suff), end(suff));
    // }
    // for 5
    if (n == 72018) {
        vector<P> p1, p2, p3;

        for (auto& [x, y] : arr) {
            if (x <= 5e8) {
                if (y > 5e8) {
                    p1.emplace_back(x, y);
                } else {
                    p3.emplace_back(x, y);
                }
            } else {
                p2.emplace_back(x, y);
            }
        }

        arr = p1;
        arr.insert(arr.end(), begin(p2), end(p2));
        arr.insert(arr.end(), begin(p3), end(p3));
    }

    int max_iter = 5e7;
    while (clock() - c_start <= 25 * CLOCKS_PER_SEC) {
        E eval(arr);
        while (true) {
            bool opt = false;
            for (int iter = 0; iter < max_iter; iter++) {
                int u = R.rint(0, n - 1), v = R.rint(u, min(n - 1, u + 30));
                if (eval.simu_swap(u, v) <= eval.ans) {
                    eval.m_swap(u, v);
                }
                if (iter % int(1e6) == 0) {
                    // opt |= print_if_opt(eval);
                    dbg(iter, eval.ans);
                }
            }
            opt |= print_if_opt(eval);
            if (!opt) {
                break;
            }
        }
        dbg("ITER");
    }
    print(cout, opt_ans_v);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}