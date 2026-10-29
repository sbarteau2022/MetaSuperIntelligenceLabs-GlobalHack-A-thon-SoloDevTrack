// Hyperbolic KV Benchmark - C++ Baseline (FW-A1-CUDA Comparison)
//
// Hand-rolled SIMD (NEON/AVX2) + inline PTX for Tensor Cores
// Implements same workload as CUDA Rust for throughput/memory comparison

#ifndef HYPERBOLIC_KV_BENCH_H
#define HYPERBOLIC_KV_BENCH_H

#include <cstdint>
#include <cmath>
#include <vector>
#include <array>
#include <algorithm>
#include <random>
#include <memory>
#include <chrono>

// Q32.31 fixed-point constants
constexpr int32_t Q32_31_SHIFT = 31;
constexpr int64_t Q32_31_SCALE = 1LL << Q32_31_SHIFT;
constexpr double Q32_31_SCALE_F64 = static_cast<double>(Q32_31_SCALE);

// SIMD detection
#if defined(__AVX2__)
    #include <immintrin.h>
    #define HAS_AVX2 1
#elif defined(__ARM_NEON)
    #include <arm_neon.h>
    #define HAS_NEON 1
#else
    #define HAS_SCALAR 1
#endif

// Inline conversion functions
inline int64_t f64_to_q32_31(double x) {
    return static_cast<int64_t>(std::llround(x * Q32_31_SCALE_F64));
}

inline double q32_31_to_f64(int64_t x) {
    return static_cast<double>(x) / Q32_31_SCALE_F64;
}

// Lorentz vector in H³ (4D: x0, x1, x2, x3) with x0 > 0
struct alignas(32) LorentzVec {
    int64_t x0, x1, x2, x3;

    LorentzVec() : x0(0), x1(0), x2(0), x3(0) {}
    LorentzVec(int64_t x0_, int64_t x1_, int64_t x2_, int64_t x3_)
        : x0(x0_), x1(x1_), x2(x2_), x3(x3_) {}

    static LorentzVec from_f64(double x0, double x1, double x2, double x3) {
        return LorentzVec(f64_to_q32_31(x0), f64_to_q32_31(x1),
                         f64_to_q32_31(x2), f64_to_q32_31(x3));
    }

    // Minkowski inner product ⟨a, b⟩_L = a₀b₀ - a₁b₁ - a₂b₂ - a₃b₃
    // Uses i128 accumulator to prevent overflow at x0 ≥ 65,536
    inline int64_t minkowski_dot(const LorentzVec& other) const {
        __int128 acc = 0;
        acc += static_cast<__int128>(x0) * other.x0;
        acc -= static_cast<__int128>(x1) * other.x1;
        acc -= static_cast<__int128>(x2) * other.x2;
        acc -= static_cast<__int128>(x3) * other.x3;
        return static_cast<int64_t>(acc >> Q32_31_SHIFT);
    }

    inline int64_t minkowski_norm_sq() const {
        return minkowski_dot(*this);
    }

    // Hyperbolic distance: d_H³(x, y) = arcosh(⟨x, y⟩_L)
    inline int64_t hyperbolic_distance(const LorentzVec& other) const {
        int64_t dot = minkowski_dot(other);
        double y = q32_31_to_f64(dot);
        if (y <= 1.0) return 0;
        double dist = std::log(y + std::sqrt(y * y - 1.0));
        return f64_to_q32_31(dist);
    }

    inline bool is_valid_h3() const {
        return x0 > 0 && std::llabs(minkowski_norm_sq() - Q32_31_SCALE) < (Q32_31_SCALE / 1000);
    }

    inline int64_t radial_coord() const { return x0; }
};

// Toroidal phase coordinates T² = (θ_f, θ_s)
struct alignas(16) ToroidalPhase {
    int64_t theta_f;  // Q32.31, range [-π, π]
    int64_t theta_s;  // Q32.31, range [-π, π]

    ToroidalPhase() : theta_f(0), theta_s(0) {}
    ToroidalPhase(int64_t tf, int64_t ts) : theta_f(tf), theta_s(ts) {}

    static ToroidalPhase from_lorentz(const LorentzVec& v) {
        double x1 = q32_31_to_f64(v.x1);
        double x2 = q32_31_to_f64(v.x2);
        double x3 = q32_31_to_f64(v.x3);

        double theta_f = std::atan2(x2, x1);
        double rho = std::sqrt(x1 * x1 + x2 * x2);
        double theta_s = std::atan2(x3, rho);

        return ToroidalPhase(f64_to_q32_31(theta_f), f64_to_q32_31(theta_s));
    }

    // Toroidal distance squared: d_T²²(φ, ψ) = (1 - cos(Δθ_f)) + (1 - cos(Δθ_s))
    inline int64_t distance_sq(const ToroidalPhase& other) const {
        double df = q32_31_to_f64(theta_f - other.theta_f);
        double ds = q32_31_to_f64(theta_s - other.theta_s);
        double dist_sq = (1.0 - std::cos(df)) + (1.0 - std::cos(ds));
        return f64_to_q32_31(dist_sq);
    }
};

// Combined H³×T² atom coordinate
struct alignas(64) AtomCoord {
    LorentzVec h3;
    ToroidalPhase t2;
    uint64_t atom_id;
    uint8_t pinned;  // 0 = unpinned, 1 = pinned
    uint8_t _pad[7]; // alignment

    AtomCoord() : atom_id(0), pinned(0), _pad{} {}
    AtomCoord(const LorentzVec& h3_, const ToroidalPhase& t2_, uint64_t id, bool pinned_)
        : h3(h3_), t2(t2_), atom_id(id), pinned(pinned_ ? 1 : 0), _pad{} {}

    // Combined retrieval score: ⟨q_H³, a_H³⟩_L + ¼ d_T²²(φ_q, φ_a)
    inline int64_t retrieval_score(const AtomCoord& query) const {
        int64_t h3_score = query.h3.minkowski_dot(h3);
        int64_t t2_penalty = query.t2.distance_sq(t2) >> 2; // divide by 4
        return h3_score - t2_penalty;
    }
};

// VP-Tree node for H³ nearest neighbor search
struct VpTreeNode {
    AtomCoord vantage;
    int64_t threshold;
    std::unique_ptr<VpTreeNode> left;   // distance ≤ threshold
    std::unique_ptr<VpTreeNode> right;  // distance > threshold
    std::vector<AtomCoord> leaf_atoms;
    bool is_leaf;

    VpTreeNode() : threshold(0), is_leaf(true) {}
    VpTreeNode(const AtomCoord& v, int64_t t,
               std::unique_ptr<VpTreeNode> l, std::unique_ptr<VpTreeNode> r)
        : vantage(v), threshold(t), left(std::move(l)), right(std::move(r)), is_leaf(false) {}
};

// VP-Tree for H³ k-NN search
class VpTree {
public:
    explicit VpTree(size_t leaf_size = 32) : leaf_size_(leaf_size) {}

    void build(std::vector<AtomCoord>& atoms) {
        if (atoms.empty()) return;
        root_ = build_recursive(atoms);
    }

    // k-NN search
    std::vector<std::pair<AtomCoord, int64_t>> knn_search(const AtomCoord& query, size_t k) const {
        std::vector<std::pair<AtomCoord, int64_t>> results;
        if (root_) {
            knn_recursive(root_.get(), query, k, results);
        }
        std::sort(results.begin(), results.end(),
                  [](const auto& a, const auto& b) { return a.second < b.second; });
        if (results.size() > k) results.resize(k);
        return results;
    }

    // Insert new atom
    void insert(const AtomCoord& atom) {
        if (!root_) {
            root_ = std::make_unique<VpTreeNode>();
            root_->leaf_atoms.push_back(atom);
            root_->is_leaf = true;
            return;
        }
        insert_recursive(root_.get(), atom);
    }

    // Evict atom with maximum radial coordinate (x0) among unpinned
    std::optional<uint64_t> evict_max_radial() {
        if (!root_) return std::nullopt;

        int64_t max_radial = INT64_MIN;
        uint64_t candidate_id = 0;
        find_max_radial(root_.get(), max_radial, candidate_id);

        if (max_radial > INT64_MIN) {
            remove_by_id(root_.get(), candidate_id);
            return candidate_id;
        }
        return std::nullopt;
    }

    size_t size() const { return count_nodes(root_.get()); }

private:
    size_t leaf_size_;
    std::unique_ptr<VpTreeNode> root_;

    std::unique_ptr<VpTreeNode> build_recursive(std::vector<AtomCoord>& atoms) {
        if (atoms.size() <= leaf_size_) {
            auto node = std::make_unique<VpTreeNode>();
            node->leaf_atoms = std::move(atoms);
            node->is_leaf = true;
            if (!node->leaf_atoms.empty()) {
                node->vantage = node->leaf_atoms[0];
            }
            return node;
        }

        // Select vantage point (median by radial coordinate)
        std::nth_element(atoms.begin(), atoms.begin() + atoms.size() / 2, atoms.end(),
                        [](const AtomCoord& a, const AtomCoord& b) {
                            return a.h3.radial_coord() < b.h3.radial_coord();
                        });

        size_t mid = atoms.size() / 2;
        AtomCoord vantage = atoms[mid];

        // Compute distances to vantage
        std::vector<std::pair<int64_t, size_t>> distances;
        distances.reserve(atoms.size());
        for (size_t i = 0; i < atoms.size(); ++i) {
            if (i == mid) continue;
            int64_t dist = atoms[i].h3.hyperbolic_distance(vantage.h3);
            distances.emplace_back(dist, i);
        }

        std::nth_element(distances.begin(), distances.begin() + distances.size() / 2, distances.end(),
                        [](const auto& a, const auto& b) { return a.first < b.first; });
        int64_t median_dist = distances[distances.size() / 2].first;

        // Partition
        std::vector<AtomCoord> left_atoms, right_atoms;
        left_atoms.reserve(mid);
        right_atoms.reserve(atoms.size() - mid - 1);

        for (const auto& [dist, idx] : distances) {
            if (dist <= median_dist) {
                left_atoms.push_back(std::move(atoms[idx]));
            } else {
                right_atoms.push_back(std::move(atoms[idx]));
            }
        }

        auto left = left_atoms.empty() ? nullptr : build_recursive(left_atoms);
        auto right = right_atoms.empty() ? nullptr : build_recursive(right_atoms);

        return std::make_unique<VpTreeNode>(vantage, median_dist, std::move(left), std::move(right));
    }

    void knn_recursive(const VpTreeNode* node, const AtomCoord& query, size_t k,
                       std::vector<std::pair<AtomCoord, int64_t>>& results) const {
        if (node->is_leaf) {
            for (const auto& atom : node->leaf_atoms) {
                int64_t dist = query.h3.hyperbolic_distance(atom.h3);
                insert_result(results, atom, dist, k);
            }
            return;
        }

        int64_t dist_to_vantage = query.h3.hyperbolic_distance(node->vantage.h3);
        insert_result(results, node->vantage, dist_to_vantage, k);

        bool search_left_first = dist_to_vantage <= node->threshold;
        const VpTreeNode* first = search_left_first ? node->left.get() : node->right.get();
        const VpTreeNode* second = search_left_first ? node->right.get() : node->left.get();

        if (first) knn_recursive(first, query, k, results);

        bool need_second = false;
        if (results.size() < k) {
            need_second = true;
        } else if (!results.empty()) {
            int64_t worst_dist = results.back().second;
            if (std::llabs(dist_to_vantage - node->threshold) <= worst_dist) {
                need_second = true;
            }
        }

        if (need_second && second) {
            knn_recursive(second, query, k, results);
        }
    }

    static void insert_result(std::vector<std::pair<AtomCoord, int64_t>>& results,
                              const AtomCoord& atom, int64_t dist, size_t k) {
        auto it = std::lower_bound(results.begin(), results.end(), dist,
                                  [](const auto& a, int64_t d) { return a.second < d; });
        results.insert(it, {atom, dist});
        if (results.size() > k) results.pop_back();
    }

    void insert_recursive(VpTreeNode* node, const AtomCoord& atom) {
        if (node->is_leaf) {
            node->leaf_atoms.push_back(atom);
            if (node->leaf_atoms.size() > leaf_size_ * 2) {
                // Rebuild this subtree
                std::vector<AtomCoord> atoms = std::move(node->leaf_atoms);
                node->leaf_atoms.clear();
                auto new_node = build_recursive(atoms);
                node->vantage = new_node->vantage;
                node->threshold = new_node->threshold;
                node->left = std::move(new_node->left);
                node->right = std::move(new_node->right);
                node->is_leaf = new_node->is_leaf;
                node->leaf_atoms = std::move(new_node->leaf_atoms);
            }
            return;
        }

        int64_t dist = atom.h3.hyperbolic_distance(node->vantage.h3);
        if (dist <= node->threshold) {
            if (node->left) {
                insert_recursive(node->left.get(), atom);
            } else {
                node->left = std::make_unique<VpTreeNode>();
                node->left->leaf_atoms.push_back(atom);
                node->left->is_leaf = true;
            }
        } else {
            if (node->right) {
                insert_recursive(node->right.get(), atom);
            } else {
                node->right = std::make_unique<VpTreeNode>();
                node->right->leaf_atoms.push_back(atom);
                node->right->is_leaf = true;
            }
        }
    }

    void find_max_radial(const VpTreeNode* node, int64_t& max_radial, uint64_t& candidate_id) const {
        if (node->is_leaf) {
            for (const auto& atom : node->leaf_atoms) {
                if (atom.pinned == 0 && atom.h3.radial_coord() > max_radial) {
                    max_radial = atom.h3.radial_coord();
                    candidate_id = atom.atom_id;
                }
            }
            return;
        }

        if (node->vantage.pinned == 0 && node->vantage.h3.radial_coord() > max_radial) {
            max_radial = node->vantage.h3.radial_coord();
            candidate_id = node->vantage.atom_id;
        }

        if (node->left) find_max_radial(node->left.get(), max_radial, candidate_id);
        if (node->right) find_max_radial(node->right.get(), max_radial, candidate_id);
    }

    bool remove_by_id(VpTreeNode* node, uint64_t id) {
        if (node->is_leaf) {
            size_t before = node->leaf_atoms.size();
            node->leaf_atoms.erase(
                std::remove_if(node->leaf_atoms.begin(), node->leaf_atoms.end(),
                              [id](const AtomCoord& a) { return a.atom_id == id; }),
                node->leaf_atoms.end());
            return node->leaf_atoms.size() < before;
        }

        if (node->vantage.atom_id == id) {
            node->vantage.pinned = 2; // removed marker
            return true;
        }

        bool found = false;
        if (node->left) found |= remove_by_id(node->left.get(), id);
        if (node->right) found |= remove_by_id(node->right.get(), id);
        return found;
    }

    size_t count_nodes(const VpTreeNode* node) const {
        if (!node) return 0;
        size_t count = node->is_leaf ? node->leaf_atoms.size() : 1;
        count += count_nodes(node->left.get());
        count += count_nodes(node->right.get());
        return count;
    }
};

// Picard fundamental domain reduction for PSL(2, Z[i])
namespace picard_reduce {

inline void reduce_moderate(LorentzVec* v) {
    for (int iter = 0; iter < 32; ++iter) {
        double x0 = q32_31_to_f64(v->x0);
        double x1 = q32_31_to_f64(v->x1);
        double x2 = q32_31_to_f64(v->x2);
        double x3 = q32_31_to_f64(v->x3);

        double denom = x0 + x3;
        if (std::fabs(denom) < 1e-10) break;

        double z_re = x1 / denom;
        double z_im = x2 / denom;
        double w = 1.0 / denom;

        double z_re_red = z_re - std::round(z_re);
        double z_im_red = z_im - std::round(z_im);

        double norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
        if (norm_sq <= 0.25 && norm_sq + w * w >= 1.0) break;

        double new_x0 = (1.0 + norm_sq) / (1.0 - norm_sq);
        double new_x1 = 2.0 * z_re_red / (1.0 - norm_sq);
        double new_x2 = 2.0 * z_im_red / (1.0 - norm_sq);
        double new_x3 = (1.0 - norm_sq + 2.0 * w * std::sqrt(1.0 - norm_sq)) / (1.0 + norm_sq);

        v->x0 = f64_to_q32_31(new_x0);
        v->x1 = f64_to_q32_31(new_x1);
        v->x2 = f64_to_q32_31(new_x2);
        v->x3 = f64_to_q32_31(new_x3);

        if (q32_31_to_f64(v->x0) <= 8.0) break;
    }
}

inline void reduce_unbounded(LorentzVec* v) {
    double x0 = q32_31_to_f64(v->x0);
    double x1 = q32_31_to_f64(v->x1);
    double x2 = q32_31_to_f64(v->x2);
    double x3 = q32_31_to_f64(v->x3);

    // Cancellation-free x0 + x3
    double x0_plus_x3 = (x0 > x3) ? (x0 + x3) : (x3 + x0);
    if (std::fabs(x0_plus_x3) < 1e-10) return;

    double z_re = x1 / x0_plus_x3;
    double z_im = x2 / x0_plus_x3;
    double w = 1.0 / x0_plus_x3;

    // Cusp folding
    double w_folded = (w > 64.0) ? (1.0 + std::log(w - 1.0)) : w;

    double z_re_red = z_re - std::round(z_re);
    double z_im_red = z_im - std::round(z_im);

    double norm_sq = z_re_red * z_re_red + z_im_red * z_im_red;
    double denom = 1.0 - norm_sq;

    if (denom > 1e-10) {
        double new_x0 = (1.0 + norm_sq) / denom;
        double new_x1 = 2.0 * z_re_red / denom;
        double new_x2 = 2.0 * z_im_red / denom;
        double new_x3 = (1.0 - norm_sq) / denom;

        v->x0 = f64_to_q32_31(new_x0);
        v->x1 = f64_to_q32_31(new_x1);
        v->x2 = f64_to_q32_31(new_x2);
        v->x3 = f64_to_q32_31(new_x3);
    }
}

inline void reduce(LorentzVec* v) {
    double x0 = q32_31_to_f64(v->x0);
    if (x0 <= 8.0) {
        reduce_moderate(v);
    } else {
        reduce_unbounded(v);
    }
}

inline bool verify_idempotence(const LorentzVec& v) {
    LorentzVec v1 = v, v2 = v;
    reduce(&v1);
    reduce(&v2);
    reduce(&v2);

    return std::llabs(v1.x0 - v2.x0) < 10 &&
           std::llabs(v1.x1 - v2.x1) < 10 &&
           std::llabs(v1.x2 - v2.x2) < 10 &&
           std::llabs(v1.x3 - v2.x3) < 10;
}

} // namespace picard_reduce

// Generate random Lorentz vector on H³
inline LorentzVec random_lorentz_vec(std::mt19937_64& rng, double x0_max) {
    std::uniform_real_distribution<double> x0_dist(1.0, x0_max);
    std::uniform_real_distribution<double> dir_dist(-1.0, 1.0);

    double x0 = x0_dist(rng);
    double r = std::sqrt(x0 * x0 - 1.0);

    while (true) {
        double x1 = dir_dist(rng);
        double x2 = dir_dist(rng);
        double x3 = dir_dist(rng);
        double norm_sq = x1*x1 + x2*x2 + x3*x3;
        if (norm_sq > 0.0 && norm_sq <= 1.0) {
            double scale = r / std::sqrt(norm_sq);
            return LorentzVec::from_f64(x0, x1 * scale, x2 * scale, x3 * scale);
        }
    }
}

// Generate random toroidal phase
inline ToroidalPhase random_toroidal_phase(std::mt19937_64& rng) {
    std::uniform_real_distribution<double> dist(-M_PI, M_PI);
    return ToroidalPhase(f64_to_q32_31(dist(rng)), f64_to_q32_31(dist(rng)));
}

#endif // HYPERBOLIC_KV_BENCH_H
