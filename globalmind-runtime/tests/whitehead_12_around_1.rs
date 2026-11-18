use globalmind_runtime::{
    BOUNDARY_TIE_EPSILON, DEFAULT_ICOSAHEDRAL_OVERLAP_MARGIN, Digest, GaussianInt, GlobalMindStore,
    ProjectivePointMod2Plus2i, Psl2Zi, RecordId, StoreConfig, TopologyStatus,
    WHITEHEAD_CLASSIFIER_VERSION, WHITEHEAD_CONTRACT_VERSION, WHITEHEAD_SPACE_NAME,
    WhiteheadTopology, canonicalize, gamma12_generators, hyperbolic_distance, joint_well_cost,
    kuramoto_psi, lorentz_inner, lorentz_to_poincare, lorentz_to_uhs, poincare_to_lorentz,
    torus_distance_sq, uhs_to_lorentz,
};
use std::collections::BTreeSet;
use std::f64::consts::{PI, TAU};
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct IsolatedSurreal {
    child: Child,
    dir: PathBuf,
    endpoint: String,
}

impl IsolatedSurreal {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().expect("local addr").port();
        drop(listener);
        assert_ne!(port, 8000, "must never touch live SurrealDB port 8000");

        let dir = std::env::temp_dir().join(format!(
            "globalmind_whitehead_test_{}_{}",
            std::process::id(),
            port
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp surreal dir");
        let db_path = dir.join("kv");
        let endpoint = format!("http://127.0.0.1:{port}");

        let surreal_bin = if Path::new("/opt/homebrew/bin/surreal").exists() {
            "/opt/homebrew/bin/surreal"
        } else {
            "surreal"
        };

        let child = Command::new(surreal_bin)
            .args([
                "start",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--user",
                "root",
                "--pass",
                "root",
                &format!("surrealkv://{}", db_path.display()),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated SurrealDB 3.3.5");

        let instance = Self {
            child,
            dir,
            endpoint,
        };
        instance.wait_ready();
        instance
    }

    fn wait_ready(&self) {
        let host_port = self.endpoint.strip_prefix("http://").unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if std::net::TcpStream::connect(host_port).is_ok() {
                thread::sleep(Duration::from_millis(120));
                return;
            }
            thread::sleep(Duration::from_millis(40));
        }
        panic!(
            "isolated SurrealDB did not become ready at {}",
            self.endpoint
        );
    }

    fn config(&self) -> StoreConfig {
        StoreConfig {
            endpoint: self.endpoint.clone(),
            namespace: "hypermind".to_string(),
            database: "whitehead_qual_isolated".to_string(),
            username: Some("root".to_string()),
            password: Some("root".to_string()),
            allow_quarantined_core: false,
        }
    }

    fn apply_schemas(&self) {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root");
        let schema_dir = repo_root.join("hypermind-nested-kv/schema");
        for file in [
            "0001_hypermind_graph.surql",
            "0003_nested_hyperbolic_kv.surql",
            "0004_transformer_kv_tensor_pages.surql",
            "0005_universal_atom_lineage.surql",
            "0006_immutable_atom_address.surql",
            "0007_globalmind_distillation_trajectories.surql",
            "0008_superposition_holding.surql",
            "0009_immutable_tensor_kv.surql",
            "0010_native_distillation_ingress.surql",
            "0011_atomic_trajectory_cells.surql",
        ] {
            let sql = fs::read_to_string(schema_dir.join(file)).expect("read schema file");
            self.raw_sql(&sql);
        }
    }

    fn raw_sql(&self, sql: &str) -> String {
        use std::io::{Read, Write};
        let host_port = self.endpoint.strip_prefix("http://").unwrap();
        let body = format!("USE NS hypermind DB whitehead_qual_isolated; {sql}");
        let req = format!(
            "POST /sql HTTP/1.1\r\nHost: {host_port}\r\nAuthorization: Basic cm9vdDpyb290\r\nAccept: application/json\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let mut stream = std::net::TcpStream::connect(host_port).expect("connect surreal");
        stream.write_all(req.as_bytes()).expect("write sql");
        let mut resp = String::new();
        stream.read_to_string(&mut resp).expect("read sql resp");
        assert!(
            resp.starts_with("HTTP/1.1 200"),
            "Surreal HTTP status failed: {resp}"
        );
        resp
    }
}

impl Drop for IsolatedSurreal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn gate1_central_hub_and_twelve_icosahedral_peer_wells() {
    let ws = RecordId::parse("globalmind").unwrap();
    let topo = WhiteheadTopology::canonical(ws).unwrap();

    // 1. Central hub invariants
    assert_eq!(topo.hub.label, "L0_Coordinator");
    assert_eq!(topo.hub.h3_center_lorentz, [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(topo.hub.t2_phase, [0.0, 0.0]);
    assert_eq!(topo.hub.status, TopologyStatus::Specified);
    let hub_uhs = lorentz_to_uhs(topo.hub.h3_center_lorentz).unwrap();
    assert!((hub_uhs.z_re).abs() < 1e-15);
    assert!((hub_uhs.z_im).abs() < 1e-15);
    assert!((hub_uhs.w - 1.0).abs() < 1e-15);
    let hub_poincare = lorentz_to_poincare(topo.hub.h3_center_lorentz).unwrap();
    assert_eq!(hub_poincare, [0.0, 0.0, 0.0]);

    // 2. Twelve peer wells on the golden icosahedral shell
    let phi = 0.5 * (1.0 + 5.0_f64.sqrt());
    let expected_r_sph = 2.0 - phi;
    let expected_hub_dist = 2.0 * expected_r_sph.atanh();
    let inv_sqrt5 = 1.0 / 5.0_f64.sqrt();

    let mut seen_ordinals = BTreeSet::new();
    let mut seen_sectors = BTreeSet::new();
    let mut seen_labels = BTreeSet::new();
    let mut kissing_distances = Vec::new();

    for well in &topo.wells {
        assert!(seen_ordinals.insert(well.ordinal));
        assert!(seen_sectors.insert(well.t2_sector));
        assert!(seen_labels.insert(well.label.clone()));
        assert_eq!(well.ordinal, well.t2_sector);
        assert_eq!(well.status, TopologyStatus::Specified);

        // Unit direction on S^2
        let d = well.unit_direction_s2;
        let norm_s2 = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((norm_s2 - 1.0).abs() < 1e-15);

        // Poincaré ball radius r_sph = 2 - phi
        let p = well.poincare_center;
        let r_p = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r_p - expected_r_sph).abs() < 1e-15);

        // Lorentz hyperboloid sheet invariant x0^2 - ||x||^2 = 1
        let h = well.h3_center_lorentz;
        let quad = lorentz_inner(h, h);
        assert!((quad - 1.0).abs() < 1e-15);

        // Stereographic round-trip
        let p_round = lorentz_to_poincare(h).unwrap();
        for k in 0..3 {
            assert!((p_round[k] - p[k]).abs() < 1e-15);
        }

        // Uniform geodesic distance to central hub L0
        let d_hub = hyperbolic_distance(topo.hub.h3_center_lorentz, h);
        assert!((d_hub - expected_hub_dist).abs() < 1e-14);

        // Welch/Simplex icosahedral coherence: 1 antipode (-1), 5 kissing (+1/sqrt(5)), 5 next-nearest (-1/sqrt(5))
        let mut count_antipode = 0;
        let mut count_kissing = 0;
        let mut count_far = 0;
        for other in &topo.wells {
            if other.ordinal == well.ordinal {
                continue;
            }
            let o = other.unit_direction_s2;
            let dot = d[0] * o[0] + d[1] * o[1] + d[2] * o[2];
            if (dot + 1.0).abs() < 1e-12 {
                count_antipode += 1;
                assert_eq!(other.ordinal, well.antipodal_ordinal);
                assert_eq!(
                    topo.overlap_matrix[well.ordinal as usize][other.ordinal as usize],
                    0.0
                );
            } else if (dot - inv_sqrt5).abs() < 1e-12 {
                count_kissing += 1;
                assert!(well.adjacent_ordinals.contains(&other.ordinal));
                assert_eq!(
                    topo.overlap_matrix[well.ordinal as usize][other.ordinal as usize],
                    DEFAULT_ICOSAHEDRAL_OVERLAP_MARGIN
                );
                kissing_distances.push(hyperbolic_distance(h, other.h3_center_lorentz));
            } else if (dot + inv_sqrt5).abs() < 1e-12 {
                count_far += 1;
                assert!(!well.adjacent_ordinals.contains(&other.ordinal));
                assert_eq!(
                    topo.overlap_matrix[well.ordinal as usize][other.ordinal as usize],
                    0.0
                );
            } else {
                panic!("unexpected icosahedral dot product {dot}");
            }
        }
        assert_eq!(count_antipode, 1);
        assert_eq!(count_kissing, 5);
        assert_eq!(count_far, 5);

        // Resonant phase psi(phi_w) sits at the exact center of sector w
        let psi = kuramoto_psi(well.t2_center_phase);
        let expected_psi = TAU * ((well.ordinal as f64) + 0.5) / 12.0;
        assert!((psi - expected_psi).abs() < 1e-14);
        let sector = ((psi / TAU) * 12.0).floor() as u8;
        assert_eq!(sector, well.t2_sector);
    }

    assert_eq!(seen_ordinals.len(), 12);
    assert_eq!(seen_sectors.len(), 12);
    assert_eq!(seen_labels.len(), 12);
    assert_eq!(kissing_distances.len(), 60);
    let first_kiss = kissing_distances[0];
    for &d_kiss in &kissing_distances {
        assert!((d_kiss - first_kiss).abs() < 1e-14);
    }
}

#[test]
fn gate2_gamma12_subgroup_cosets_and_schreier_generators() {
    // 1. Verify |Z[i] / (2 + 2i)| = 8 and |P^1(Z[i] / (2 + 2i))| = 12
    let mut residue_codes = BTreeSet::new();
    for re in -8..=8 {
        for im in -8..=8 {
            residue_codes.insert(GaussianInt::new(re, im).mod_2_plus_2i_code());
        }
    }
    assert_eq!(residue_codes.len(), 8);
    let proj_pts = ProjectivePointMod2Plus2i::all_twelve();
    assert_eq!(proj_pts.len(), 12);

    // 2. Verify the 12 left-coset representatives C_0 .. C_11
    let reps = Psl2Zi::coset_representatives();
    let mut coset_indices = BTreeSet::new();
    for (k, &rep) in reps.iter().enumerate() {
        assert_eq!(rep.determinant(), GaussianInt::ONE);
        assert_eq!(rep.left_coset_index(), k);
        assert!(coset_indices.insert(rep.left_coset_index()));
        // Only C_0 = I lies in Gamma_12
        assert_eq!(rep.is_in_gamma12(), k == 0);
    }
    assert_eq!(coset_indices.len(), 12);

    // 3. Verify that left multiplication by Picard generators {S, T1, Ti, J}
    //    permutes the 12 left cosets C_k * Gamma_12 (4 permutations in S_12)
    for g in [
        Psl2Zi::gen_s(),
        Psl2Zi::gen_t1(),
        Psl2Zi::gen_ti(),
        Psl2Zi::gen_j(),
    ] {
        let mut image_cosets = BTreeSet::new();
        for &rep in &reps {
            let target_coset = g.mul(rep).left_coset_index();
            image_cosets.insert(target_coset);
        }
        assert_eq!(
            image_cosets.len(),
            12,
            "left multiplication by {g:?} must induce a bijection on the 12 cosets"
        );
    }

    // 4. Verify Schreier + Whitehead generators of Gamma_12
    let gens = gamma12_generators();
    assert!(gens.len() >= 15);
    for g in &gens {
        assert_eq!(g.matrix.determinant(), GaussianInt::ONE);
        assert!(g.matrix.is_in_gamma12());
        assert_eq!(g.matrix.left_coset_index(), 0);
    }
}

#[test]
fn gate3_h3_action_uhs_equivalence_and_t2_coupling() {
    let gens = gamma12_generators();
    let test_points = [
        poincare_to_lorentz([0.15, -0.22, 0.11]).unwrap(),
        poincare_to_lorentz([-0.31, 0.14, 0.25]).unwrap(),
        poincare_to_lorentz([0.08, 0.33, -0.19]).unwrap(),
        poincare_to_lorentz([0.0, 0.0, 0.0]).unwrap(),
    ];

    // 1. Verify Hermitian spinor action X' = M X M^\dagger agrees with 3D Poincaré UHS extension
    //    and preserves Lorentz inner products <M.x, M.y>_L = <x, y>_L
    for g in &gens {
        let m = g.matrix;
        let a = (m.a.re as f64, m.a.im as f64);
        let b = (m.b.re as f64, m.b.im as f64);
        let c = (m.c.re as f64, m.c.im as f64);
        let d = (m.d.re as f64, m.d.im as f64);

        for &x in &test_points {
            let hx = m.act_h3(x).unwrap();
            let u = lorentz_to_uhs(x).unwrap();
            let z = (u.z_re, u.z_im);
            let w = u.w;

            // (az + b), (cz + d)
            let az_b = (a.0 * z.0 - a.1 * z.1 + b.0, a.0 * z.1 + a.1 * z.0 + b.1);
            let cz_d = (c.0 * z.0 - c.1 * z.1 + d.0, c.0 * z.1 + c.1 * z.0 + d.1);
            let c_norm_sq = c.0 * c.0 + c.1 * c.1;
            let denom = cz_d.0 * cz_d.0 + cz_d.1 * cz_d.1 + c_norm_sq * w * w;
            // (az + b) * conj(cz + d) + a * conj(c) * w^2
            let prod1 = (
                az_b.0 * cz_d.0 + az_b.1 * cz_d.1,
                -az_b.0 * cz_d.1 + az_b.1 * cz_d.0,
            );
            let a_conj_c = (a.0 * c.0 + a.1 * c.1, -a.0 * c.1 + a.1 * c.0);
            let z_next = (
                (prod1.0 + a_conj_c.0 * w * w) / denom,
                (prod1.1 + a_conj_c.1 * w * w) / denom,
            );
            let w_next = w / denom;
            let uhs_lorentz = uhs_to_lorentz(globalmind_runtime::UhsCoord {
                z_re: z_next.0,
                z_im: z_next.1,
                w: w_next,
            })
            .unwrap();

            assert!(
                hyperbolic_distance(hx, uhs_lorentz) < 1e-12,
                "Hermitian spinor action and 3D UHS Poincaré extension diverged for {}",
                g.name
            );
        }

        for i in 0..test_points.len() {
            for j in i..test_points.len() {
                let x = test_points[i];
                let y = test_points[j];
                let mx = m.act_h3(x).unwrap();
                let my = m.act_h3(y).unwrap();
                let ip_before = lorentz_inner(x, y);
                let ip_after = lorentz_inner(mx, my);
                assert!(
                    (ip_before - ip_after).abs() < 1e-11,
                    "Lorentz inner product not preserved under {}: {ip_before} vs {ip_after}",
                    g.name
                );
            }
        }
    }

    // 2. Verify that the character map Gamma_12 -> (Z_2)^3 is surjective (all 8 classes realized)
    //    and that the coupled T^2 shift preserves the 2:1 Kuramoto invariant psi
    let mut character_classes = BTreeSet::new();
    for g1 in &gens {
        character_classes.insert(g1.matrix.gamma12_character_bits().unwrap());
        for g2 in &gens {
            let prod = g1.matrix.mul(g2.matrix);
            character_classes.insert(prod.gamma12_character_bits().unwrap());

            let x = test_points[0];
            let phi = [1.234, 2.345];
            let (x1, phi1) = g2.matrix.act_coupled_gamma12(x, phi).unwrap();
            let (x12, phi12) = g1.matrix.act_coupled_gamma12(x1, phi1).unwrap();
            let (xp, phip) = prod.act_coupled_gamma12(x, phi).unwrap();
            assert!(hyperbolic_distance(x12, xp) < 1e-11);
            assert!(torus_distance_sq(phi12, phip) < 1e-12);
            assert!((kuramoto_psi(phip) - kuramoto_psi(phi)).abs() < 1e-12);
        }
    }
    assert_eq!(
        character_classes.len(),
        8,
        "Gamma_12 character homomorphism onto (Z_2)^3 must realize all 8 classes"
    );

    // 3. Verify that stepping a point in F_Gamma through the 12 coset tiles C_k^-1 . x0
    //    bijectively visits all 12 cosets and all 12 coupled T^2 sectors!
    let base_x = uhs_to_lorentz(globalmind_runtime::UhsCoord {
        z_re: 0.11,
        z_im: 0.17,
        w: 1.45,
    })
    .unwrap();
    let base_phi = [0.2, 0.05];
    let reps = Psl2Zi::coset_representatives();
    let mut visited_cosets = BTreeSet::new();
    let mut visited_sectors = BTreeSet::new();
    for &rep in &reps {
        let tile_x = rep.inv().act_h3(base_x).unwrap();
        let can = canonicalize(tile_x, base_phi).unwrap();
        visited_cosets.insert(can.coset_index);
        visited_sectors.insert(can.coupled_sector);
    }
    assert_eq!(visited_cosets.len(), 12);
    assert_eq!(visited_sectors.len(), 12);
}

#[test]
fn gate4_canonical_representative_idempotence_and_orbit_invariance() {
    let gens = gamma12_generators();

    // Test 240 deterministic points across multiple radii and directions
    for idx in 0..240usize {
        let r = 0.05 + 0.80 * ((idx % 12) as f64 / 11.0);
        let theta = TAU * ((idx * 7) as f64 / 240.0);
        let z_ang = PI * (((idx * 13) % 240) as f64 / 240.0) - 0.5 * PI;
        let p = [
            r * z_ang.cos() * theta.cos(),
            r * z_ang.cos() * theta.sin(),
            r * z_ang.sin(),
        ];
        let x = poincare_to_lorentz(p).unwrap();
        let phi = [
            (0.37 * (idx as f64)).rem_euclid(TAU),
            (0.19 * (idx as f64)).rem_euclid(TAU),
        ];

        let can1 = canonicalize(x, phi).unwrap();
        // Verify Picard fundamental domain bounds
        let u_pic = lorentz_to_uhs(can1.h3_picard).unwrap();
        assert!(u_pic.z_re >= -0.5 - 1e-9 && u_pic.z_re <= 0.5 + 1e-9);
        assert!(u_pic.z_im >= -1e-9 && u_pic.z_im <= 0.5 + 1e-9);
        assert!(u_pic.norm_sq() >= 1.0 - 1e-8);

        // Idempotence
        let can2 = canonicalize(can1.h3_gamma12, can1.t2_gamma12).unwrap();
        assert!(hyperbolic_distance(can1.h3_gamma12, can2.h3_gamma12) < 1e-10);
        assert!(torus_distance_sq(can1.t2_gamma12, can2.t2_gamma12) < 1e-11);
        assert_eq!(can1.coset_index, can2.coset_index);
        assert_eq!(can1.coupled_sector, can2.coupled_sector);

        // Orbit invariance under Gamma_12 generators
        if idx < 24 {
            for g in &gens {
                let (xg, phig) = g.matrix.act_coupled_gamma12(x, phi).unwrap();
                let cang = canonicalize(xg, phig).unwrap();
                assert!(hyperbolic_distance(can1.h3_gamma12, cang.h3_gamma12) < 1e-9);
                assert!(torus_distance_sq(can1.t2_gamma12, cang.t2_gamma12) < 1e-10);
                assert_eq!(can1.coset_index, cang.coset_index);
                assert_eq!(can1.coupled_sector, cang.coupled_sector);
            }
        }
    }
}

#[test]
fn gate5_boundary_tie_breaking_and_icosahedral_overlap_policy() {
    let ws = RecordId::parse("globalmind").unwrap();
    let topo = WhiteheadTopology::canonical(ws).unwrap();

    // 1. For every icosahedrally adjacent pair (r, s) with r < s, locate the exact
    //    Voronoi boundary point along the Poincare segment p(t) = (1 - t) * p_r + t * p_s
    //    (at higher shell radius r_test = 0.85 so H3 distance dominates T2 differences)
    //    where cost_r(x(t*), mid_phi) == cost_s(x(t*), mid_phi) and verify that
    //    (r, s) are tied for 1st place, lowest ordinal r wins, and s is in authorized_overlap_ordinals.
    for r in 0..12usize {
        for &s_u8 in &topo.wells[r].adjacent_ordinals {
            let s = s_u8 as usize;
            if r >= s {
                continue;
            }
            let dr = topo.wells[r].unit_direction_s2;
            let ds = topo.wells[s].unit_direction_s2;
            let mid_dir = [dr[0] + ds[0], dr[1] + ds[1], dr[2] + ds[2]];
            let mid_norm =
                (mid_dir[0] * mid_dir[0] + mid_dir[1] * mid_dir[1] + mid_dir[2] * mid_dir[2])
                    .sqrt();
            // Point at Poincare radius 0.96 along the bisector ray between d_r and d_s:
            // equidistant in H3 to c_r and c_s, and > 1.5 hyperbolic units closer in H3
            // to {r, s} than to any of the other 10 wells (dominating the <= 1.0 T2 term).
            let bisector_p = [
                0.96 * mid_dir[0] / mid_norm,
                0.96 * mid_dir[1] / mid_norm,
                0.96 * mid_dir[2] / mid_norm,
            ];
            let phir = topo.wells[r].t2_center_phase;
            let phis = topo.wells[s].t2_center_phase;
            let short_mid = |a: f64, b: f64| -> f64 {
                let m1 = (0.5 * (a + b)).rem_euclid(TAU);
                let m2 = (m1 + PI).rem_euclid(TAU);
                if (m1 - a).cos() >= (m2 - a).cos() {
                    m1
                } else {
                    m2
                }
            };
            let mid_phi = [short_mid(phir[0], phis[0]), short_mid(phir[1], phis[1])];
            let mid_x = poincare_to_lorentz(bisector_p).unwrap();

            let cost_r = joint_well_cost(
                mid_x,
                mid_phi,
                topo.wells[r].h3_center_lorentz,
                topo.wells[r].t2_center_phase,
            );
            let cost_s = joint_well_cost(
                mid_x,
                mid_phi,
                topo.wells[s].h3_center_lorentz,
                topo.wells[s].t2_center_phase,
            );
            assert!((cost_r - cost_s).abs() < 1e-14);

            let cls = topo.classify_direct_frame(mid_x, mid_phi).unwrap();
            assert_eq!(cls.primary_ordinal, r as u8);
            assert_eq!(cls.runner_up_ordinal, s as u8);
            assert!(cls.on_equidistant_boundary);
            assert!(cls.tie_broken_by_lowest_ordinal);
            assert!(cls.boundary_margin <= BOUNDARY_TIE_EPSILON);
            assert!(
                cls.authorized_overlap_ordinals.contains(&(s as u8)),
                "adjacent well {s} must be in authorized_overlap_ordinals of well {r}"
            );
        }
    }

    // 2. Verify that an exact tie between two ANTIPODAL wells (r, r ^ 1) breaks ties by
    //    lowest ordinal and NEVER grants overlap authorization (since Omega_{r, antipode} == 0).
    let origin_x = [1.0, 0.0, 0.0, 0.0];
    // For w=0 (sector_angle = pi/12) and w=6 (sector_angle = 13*pi/12 = pi/12 + pi),
    // choose phi orthogonal to both so w=0 and w=6 have equal minimum torus distance.
    let cls_origin = topo.classify_direct_frame(origin_x, [0.0, 0.0]).unwrap();
    for &ov in &cls_origin.authorized_overlap_ordinals {
        assert!(
            topo.wells[cls_origin.primary_ordinal as usize]
                .adjacent_ordinals
                .contains(&ov),
            "non-adjacent well {ov} must never enter authorized_overlap_ordinals"
        );
    }
}

#[test]
fn gate6_no_gap_no_overlap_and_gamma12_invariance_across_1440_states() {
    let ws = RecordId::parse("globalmind").unwrap();
    let topo = WhiteheadTopology::canonical(ws).unwrap();
    let gens = gamma12_generators();

    // 1. Every well center classifies to itself with cost 0.0 under both classify_well
    //    and classify_direct_frame, and remains invariant under Gamma_12 orbits.
    for well in &topo.wells {
        let cls_q = topo
            .classify_well(well.h3_center_lorentz, well.t2_center_phase)
            .unwrap();
        assert_eq!(cls_q.primary_ordinal, well.ordinal);
        assert_eq!(cls_q.primary_well_id, well.id);
        assert!(cls_q.primary_cost < 1e-13);
        assert!(!cls_q.on_equidistant_boundary);
        assert!(cls_q.boundary_margin > 0.01);

        let cls_d = topo
            .classify_direct_frame(well.h3_center_lorentz, well.t2_center_phase)
            .unwrap();
        assert_eq!(cls_d.primary_ordinal, well.ordinal);
        assert!(cls_d.primary_cost < 1e-13);

        for g in gens.iter().take(8) {
            let (xg, phig) = g
                .matrix
                .act_coupled_gamma12(well.h3_center_lorentz, well.t2_center_phase)
                .unwrap();
            let cls_g = topo.classify_well(xg, phig).unwrap();
            assert_eq!(cls_g.primary_ordinal, well.ordinal);
            assert!(cls_g.primary_cost < 1e-10);
        }
    }

    // 2. Sweep 1,440 states (10 radial shells x 12 icosahedral rays x 12 phase centers)
    let mut well_counts = [0usize; 12];
    let mut direct_well_counts = [0usize; 12];
    for shell in 0..10usize {
        let r = 0.06 + 0.08 * (shell as f64);
        for dir_idx in 0..12usize {
            let dir = topo.wells[dir_idx].unit_direction_s2;
            let x = poincare_to_lorentz([r * dir[0], r * dir[1], r * dir[2]]).unwrap();
            for phase_idx in 0..12usize {
                let phi = topo.wells[phase_idx].t2_center_phase;
                let cls = topo.classify_well(x, phi).unwrap();
                assert!(cls.primary_ordinal < 12);
                assert!(cls.primary_cost.is_finite() && cls.primary_cost >= 0.0);
                assert_eq!(cls.classifier_version, WHITEHEAD_CLASSIFIER_VERSION);
                well_counts[cls.primary_ordinal as usize] += 1;

                let cls_dir = topo.classify_direct_frame(x, phi).unwrap();
                assert!(cls_dir.primary_ordinal < 12);
                direct_well_counts[cls_dir.primary_ordinal as usize] += 1;
            }
        }
    }

    let total_classified: usize = well_counts.iter().sum();
    assert_eq!(total_classified, 1440);
    for (w, &count) in well_counts.iter().enumerate() {
        assert!(
            count > 0,
            "quotient classifier must populate every well (well {w} had 0)"
        );
    }
    let total_direct: usize = direct_well_counts.iter().sum();
    assert_eq!(total_direct, 1440);
    for (w, &count) in direct_well_counts.iter().enumerate() {
        assert!(
            (118..=122).contains(&count),
            "direct icosahedral-toroidal grid must distribute 1440 states near-uniformly (~120 per well, well {w} got {count})"
        );
    }
}

#[test]
fn gate7_surrealdb_3_3_5_topology_and_membership_persistence() {
    let surreal = IsolatedSurreal::start();
    surreal.apply_schemas();
    surreal.raw_sql(
        "UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;",
    );

    let store = GlobalMindStore::connect(surreal.config()).expect("connect store");
    store.verify_contract().expect("verify GlobalMind contract");

    let ws = RecordId::parse("globalmind").unwrap();
    let topo = WhiteheadTopology::canonical(ws.clone()).unwrap();
    assert_eq!(topo.contract_version, WHITEHEAD_CONTRACT_VERSION);

    // 1. Admit topology (1 geometry_space, 1 cognitive_hub, 12 cognitive_well, 12 hub_has_well)
    store
        .admit_whitehead_topology(&topo)
        .expect("admit Whitehead 12-around-1 topology");
    store
        .verify_whitehead_topology(&topo)
        .expect("verify persisted Whitehead 12-around-1 topology");

    // 2. Classify 12 addresses (one per well) and persist `address_well_membership`
    for well in &topo.wells {
        let atom_id = format!("atom_well_{:02}", well.ordinal);
        let addr_id = RecordId::parse(format!("addr_well_{:02}", well.ordinal)).unwrap();
        let mem_id = RecordId::parse(format!("mem_well_{:02}", well.ordinal)).unwrap();
        let digest = Digest::sha256(atom_id.as_bytes());
        let h3 = well.h3_center_lorentz;
        let t2 = well.t2_center_phase;

        surreal.raw_sql(&format!(
            "CREATE atom:`{atom_id}` CONTENT {{ workspace: workspace:globalmind, external_id: '{atom_id}', content_kind: 'text', content_text: 'Whitehead well {} anchor', content_uri: NONE, content_digest: '{}', domain: 'geometry', rank: 0, salience: 0.9, provenance: 'whitehead_test', taint: 'clean', pinned: true, current_address: atom_address:`{}`, created_at: time::now(), updated_at: time::now() }} RETURN NONE; \
             CREATE atom_address:`{}` CONTENT {{ workspace: workspace:globalmind, atom: atom:`{atom_id}`, h3_lorentz: [{:?}, {:?}, {:?}, {:?}], t2_phase: [{:?}, {:?}], geometry_model: 'lorentz-h3-t2', geometry_version: 'v1', revision: 1, active: true, space: geometry_space:`{}`, created_at: time::now() }} RETURN NONE;",
            well.ordinal,
            digest.as_str(),
            addr_id.as_str(),
            addr_id.as_str(),
            h3[0],
            h3[1],
            h3[2],
            h3[3],
            t2[0],
            t2[1],
             topo.geometry_space_id.as_str(),
        ));

        let cls = topo.classify_well(h3, t2).unwrap();
        assert_eq!(cls.primary_ordinal, well.ordinal);
        store
            .record_address_well_membership(&mem_id, &ws, &addr_id, &cls)
            .expect("record address_well_membership");
    }

    // 3. Verify that SurrealDB 3.3.5 rejects out-of-range well ordinals (12) and duplicate sectors
    let bad_ordinal_resp = surreal.raw_sql(&format!(
        "CREATE cognitive_well:bad_ord_12 CONTENT {{ workspace: workspace:globalmind, geometry_space: geometry_space:`{}`, ordinal: 12, label: 'Invalid13thWell', h3_center_lorentz: [1.0, 0.0, 0.0, 0.0], t2_sector: 0, boundary_contract_digest: 'bad', status: 'specified', created_at: time::now() }};",
        topo.geometry_space_id.as_str()
    ));
    assert!(
        bad_ordinal_resp.contains("\"status\":\"ERR\""),
        "SurrealDB must reject ordinal = 12 on cognitive_well: {bad_ordinal_resp}"
    );

    let dup_sector_resp = surreal.raw_sql(&format!(
        "CREATE cognitive_well:dup_sector_0 CONTENT {{ workspace: workspace:globalmind, geometry_space: geometry_space:`{}`, ordinal: 0, label: 'DuplicateWell0', h3_center_lorentz: [1.0, 0.0, 0.0, 0.0], t2_sector: 0, boundary_contract_digest: 'dup', status: 'specified', created_at: time::now() }};",
        topo.geometry_space_id.as_str()
    ));
    assert!(
        dup_sector_resp.contains("\"status\":\"ERR\""),
        "SurrealDB must reject duplicate (workspace, geometry_space, ordinal/t2_sector): {dup_sector_resp}"
    );

    // 4. Print complete machine-computed summary for report verification
    let gens = gamma12_generators();
    println!(
        "WHITEHEAD_REPORT space={} contract={} contract_digest={} topology_digest={} classifier_digest={} hub_id={} wells={} cosets=12 schreier_generators={}",
        WHITEHEAD_SPACE_NAME,
        topo.contract_version,
        topo.contract_digest.as_str(),
        topo.topology_contract_digest.as_str(),
        topo.classifier_digest.as_str(),
        topo.hub.id.as_str(),
        topo.wells.len(),
        gens.len()
    );
    for w in &topo.wells {
        let c = w.coset_representative;
        println!(
            "WELL {:02} label={} axis={} pole={} h3=[{:.12}, {:.12}, {:.12}, {:.12}] t2_sector={} t2_center=[{:.12}, {:.12}] adj={:?} antipode={} coset=[[{}+{}i, {}+{}i], [{}+{}i, {}+{}i]] digest={}",
            w.ordinal,
            w.label,
            w.duality_axis,
            w.pole.as_str(),
            w.h3_center_lorentz[0],
            w.h3_center_lorentz[1],
            w.h3_center_lorentz[2],
            w.h3_center_lorentz[3],
            w.t2_sector,
            w.t2_center_phase[0],
            w.t2_center_phase[1],
            w.adjacent_ordinals,
            w.antipodal_ordinal,
            c.a.re,
            c.a.im,
            c.b.re,
            c.b.im,
            c.c.re,
            c.c.im,
            c.d.re,
            c.d.im,
            w.boundary_contract_digest.as_str()
        );
    }
    for (idx, g) in gens.iter().enumerate() {
        let m = g.matrix;
        println!(
            "GEN {:02} name={} matrix=[[{}+{}i, {}+{}i], [{}+{}i, {}+{}i]] char_bits={:?}",
            idx,
            g.name,
            m.a.re,
            m.a.im,
            m.b.re,
            m.b.im,
            m.c.re,
            m.c.im,
            m.d.re,
            m.d.im,
            g.character_bits
        );
    }
}
