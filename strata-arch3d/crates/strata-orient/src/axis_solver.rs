use crate::fracture_seg::SherdSegmentation;
use serde::{Deserialize, Serialize};
use strata_core::{HandleAttachmentKind, Mat4, SurfaceRegion, TriMesh};

/// Huber IRLS 법선 공면성 기반 회전축 및 물레속선(`Rills`) 경사각 복원 결과.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RotationalAxisEstimate {
    pub axis_point: [f64; 3],
    pub axis_dir: [f64; 3],
    /// 수직 Z축 대비 기울기 보정 각도 (도)
    pub tilt_correction_deg: f64,
    /// 동체부 소편 물레속선(`Rills`) 수평 정렬 경사각 (도, `0.1°` 정밀도)
    pub rills_stance_deg: f64,
    pub normal_coplanarity_rmse: f64,
}

/// 파단면(`FractureZone`)이 배제된 내·외벽 정점과 법선을 사용하여
/// Huber 가중치($\delta = 1.345\sigma$) 반복 재가중 최소제곱법(IRLS)으로 회전축 $(\mathbf{p}_0, \mathbf{a})$를 추론한다.
pub fn solve_rotational_axis_huber_irls(
    mesh: &TriMesh,
    seg: &SherdSegmentation,
) -> RotationalAxisEstimate {
    let center = mesh.bbox.center();
    let mut axis_point = [center[0], center[1], 0.0];
    let mut axis_dir = [0.0f64, 0.0, 1.0];

    // Collect non-fracture (position, normal) pairs
    let wall_samples: Vec<([f64; 3], [f64; 3])> = mesh
        .positions
        .iter()
        .zip(&mesh.normals)
        .zip(&seg.vertex_regions)
        .filter_map(|((&p, &n), &r)| {
            if r != SurfaceRegion::FractureZone {
                Some((p, n))
            } else {
                None
            }
        })
        .collect();

    if wall_samples.len() < 6 {
        return RotationalAxisEstimate {
            axis_point,
            axis_dir,
            tilt_correction_deg: 0.0,
            rills_stance_deg: 0.0,
            normal_coplanarity_rmse: 0.0,
        };
    }

    // Estimate XY axis center by normal intersection in XY plane with Huber IRLS
    for _iter in 0..15 {
        let mut a00 = 0.0;
        let mut a01 = 0.0;
        let mut a11 = 0.0;
        let mut b0 = 0.0;
        let mut b1 = 0.0;

        for &(p, n) in &wall_samples {
            // Line perpendicular to normal in XY: (-ny) * (x - px) + nx * (y - py) = 0
            let lx = -n[1];
            let ly = n[0];
            let norm_xy = (lx * lx + ly * ly).sqrt();
            if norm_xy < 0.2 {
                continue;
            }
            let ux = lx / norm_xy;
            let uy = ly / norm_xy;
            let res = ux * (axis_point[0] - p[0]) + uy * (axis_point[1] - p[1]);
            let w = if res.abs() <= 2.5 {
                1.0
            } else {
                2.5 / res.abs()
            };
            let rhs = ux * p[0] + uy * p[1];
            a00 += w * ux * ux;
            a01 += w * ux * uy;
            a11 += w * uy * uy;
            b0 += w * ux * rhs;
            b1 += w * uy * rhs;
        }

        let det = a00 * a11 - a01 * a01;
        if det.abs() > 1e-9 {
            axis_point[0] = (a11 * b0 - a01 * b1) / det;
            axis_point[1] = (a00 * b1 - a01 * b0) / det;
        }
    }

    // Estimate axis tilt (ax, ay, 1.0) minimizing [(p - p0) x a] . n = 0
    let mut m00 = 0.0;
    let mut m01 = 0.0;
    let mut m11 = 0.0;
    let mut r0 = 0.0;
    let mut r1 = 0.0;

    for &(p, n) in &wall_samples {
        let dx = p[0] - axis_point[0];
        let dy = p[1] - axis_point[1];
        let dz = p[2] - center[2];
        // Cross product ((dx, dy, dz) x (ax, ay, 1)) . (nx, ny, nz) = 0
        // => ax * (dz*ny - dy*nz) + ay * (dx*nz - dz*nx) + (dy*nx - dx*ny) = 0
        let c_x = dz * n[1] - dy * n[2];
        let c_y = dx * n[2] - dz * n[0];
        let c_z = dy * n[0] - dx * n[1];
        let w = 1.0 / (1.0 + c_z.abs() * 0.2);
        m00 += w * c_x * c_x;
        m01 += w * c_x * c_y;
        m11 += w * c_y * c_y;
        r0 -= w * c_x * c_z;
        r1 -= w * c_y * c_z;
    }

    let det_t = m00 * m11 - m01 * m01;
    if det_t.abs() > 1e-9 {
        let ax = ((m11 * r0 - m01 * r1) / det_t).clamp(-0.5, 0.5);
        let ay = ((m00 * r1 - m01 * r0) / det_t).clamp(-0.5, 0.5);
        let len = (ax * ax + ay * ay + 1.0).sqrt();
        axis_dir = [ax / len, ay / len, 1.0 / len];
    }

    let tilt_correction_deg = axis_dir[2].clamp(-1.0, 1.0).acos().to_degrees();
    let rills_stance_deg = recover_body_sherd_stance_from_rills(mesh, seg);

    let mut sq_sum = 0.0;
    for &(p, n) in &wall_samples {
        let dx = p[0] - axis_point[0];
        let dy = p[1] - axis_point[1];
        let dz = p[2] - axis_point[2];
        let cx = dy * axis_dir[2] - dz * axis_dir[1];
        let cy = dz * axis_dir[0] - dx * axis_dir[2];
        let cz = dx * axis_dir[1] - dy * axis_dir[0];
        let dot = cx * n[0] + cy * n[1] + cz * n[2];
        sq_sum += dot * dot;
    }
    let normal_coplanarity_rmse = (sq_sum / (wall_samples.len() as f64)).sqrt();

    RotationalAxisEstimate {
        axis_point,
        axis_dir,
        tilt_correction_deg,
        rills_stance_deg,
        normal_coplanarity_rmse,
    }
}

/// 동체부 소편(`BodySherd`)의 내·외면에 남은 물레 회전 물손질흔(`Rills`) 방향 벡터를 추출하여
/// 물레속선이 수평면($Z=\text{const}$)과 평행해지도록 하는 경사각(`0.1°` 정밀도)을 복원한다.
pub fn recover_body_sherd_stance_from_rills(mesh: &TriMesh, seg: &SherdSegmentation) -> f64 {
    let mut weighted_angle_sum = 0.0;
    let mut weight_total = 0.0;

    for (t_idx, tri) in mesh.triangles.iter().enumerate() {
        if seg.face_regions[t_idx] == SurfaceRegion::FractureZone {
            continue;
        }
        let idxs = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for k in 0..3 {
            let u = idxs[k];
            let v = idxs[(k + 1) % 3];
            let dk = (mesh.mean_curvatures[u] - mesh.mean_curvatures[v]).abs();
            // Rill striae follow iso-curvature contours (small dk along the edge, high curvature magnitude)
            let k_mag = 0.5 * (mesh.mean_curvatures[u].abs() + mesh.mean_curvatures[v].abs());
            if k_mag > 1e-4 {
                let pu = mesh.positions[u];
                let pv = mesh.positions[v];
                let dx = pv[0] - pu[0];
                let dy = pv[1] - pu[1];
                let dz = pv[2] - pu[2];
                let horiz = (dx * dx + dy * dy).sqrt();
                if horiz > 1e-5 {
                    let slope_deg = (dz / horiz).atan().to_degrees();
                    if slope_deg.abs() <= 45.0 {
                        let iso_weight = k_mag / (dk + 1e-3);
                        weighted_angle_sum += iso_weight * slope_deg;
                        weight_total += iso_weight;
                    }
                }
            }
        }
    }

    if weight_total > 1e-9 {
        let raw_deg = weighted_angle_sum / weight_total;
        (raw_deg * 10.0).round() / 10.0
    } else {
        0.0
    }
}

/// 특수 부위 편조각(파수·주구·뚜껑·고배 투창편) 기하 특징 판별 결과.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecialPartDiagnostics {
    /// 파수·주구 접합 방식 (`접합식` vs `투공 삽입촉`)
    pub handle_attachment: HandleAttachmentKind,
    /// 뚜껑(개, Lid) 여부 및 꼭지 상단 정위치 필요 여부
    pub is_lid_knob_on_top: bool,
    /// 고배 대각 투창(구멍) 칼질 절단 열린 면(2줄 실선 표기 대상) 개수
    pub fenestrated_open_cut_count: usize,
}

pub fn analyze_special_vessel_part(mesh: &TriMesh) -> SpecialPartDiagnostics {
    // 내부로 돌출된 촉(Tenon) 곡률 급변부가 있으면 삽입식(InsertedTenonJoint), 없으면 부착식(SurfaceButtJoint)
    let min_k = mesh.mean_curvatures.iter().copied().fold(0.0f64, f64::min);
    let handle_attachment = if min_k < -0.35 {
        HandleAttachmentKind::InsertedTenonJoint
    } else {
        HandleAttachmentKind::SurfaceButtJoint
    };

    // 상단 중앙부(`r < 0.25 * R_max`, `z > 0.75 * Z_max`)에 꼭지(Knob) 정점이 밀집하면 뚜껑(개) 정위치
    let center = mesh.bbox.center();
    let ext = mesh.bbox.extents();
    let r_knob = (ext[0].max(ext[1]) * 0.18).max(5.0);
    let z_thresh = mesh.bbox.min[2] + ext[2] * 0.75;
    let top_center_count = mesh
        .positions
        .iter()
        .filter(|p| {
            let r2 = (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2);
            p[2] >= z_thresh && r2 <= r_knob * r_knob
        })
        .count();

    // 고배 투창 칼질 절단면: 중간 높이 대역(`0.2 < z_norm < 0.8`)의 매끄러운 수직/사선 경계 루프 검출
    let mut mid_boundary_corners = 0usize;
    let z_low = mesh.bbox.min[2] + ext[2] * 0.20;
    let z_high = mesh.bbox.min[2] + ext[2] * 0.80;
    for c in 0..(mesh.triangles.len() * 3) {
        if mesh.topology.is_boundary_corner(c) {
            let v_idx = mesh.triangles[c / 3][c % 3] as usize;
            let z = mesh.positions[v_idx][2];
            if z >= z_low && z <= z_high {
                mid_boundary_corners += 1;
            }
        }
    }
    let fenestrated_open_cut_count = mid_boundary_corners / 12;

    SpecialPartDiagnostics {
        handle_attachment,
        is_lid_knob_on_top: top_center_count >= 8,
        fenestrated_open_cut_count,
    }
}

/// 유물편(`Potsherd`)의 3×3 정점 공분산 행렬 고유분해($\lambda_1 \ge \lambda_2 \ge \lambda_3$)를 통해
/// 최소 분산 두께 축($\mathbf{e}_3$)을 정면 시선 축(`Y`), 폭 축($\mathbf{e}_2$)을 수평 축(`X`),
/// 최대 길이·높이 축($\mathbf{e}_1$)을 수직 축(`+Z`)으로 정렬하여 바닥에 누운 파편을 똑바로 세운다 (`Task A-1` & `A6`).
pub fn orient_potsherd_pca_upright(mesh: &TriMesh) -> TriMesh {
    if mesh.positions.len() < 4 || mesh.ap2_cut_offset_mm.is_some() {
        return mesh.clone();
    }

    let n = mesh.positions.len() as f64;
    let mut mean = [0.0_f64; 3];
    for p in &mesh.positions {
        mean[0] += p[0];
        mean[1] += p[1];
        mean[2] += p[2];
    }
    mean[0] /= n;
    mean[1] /= n;
    mean[2] /= n;

    let mut cov = [[0.0_f64; 3]; 3];
    for p in &mesh.positions {
        let d = [p[0] - mean[0], p[1] - mean[1], p[2] - mean[2]];
        for r in 0..3 {
            for c in 0..3 {
                cov[r][c] += d[r] * d[c];
            }
        }
    }
    for row in &mut cov {
        for val in row {
            *val /= n;
        }
    }

    // 3x3 대칭 행렬 Jacobi 고유분해
    let mut v = [[1.0_f64, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut a = cov;
    for _sweep in 0..25 {
        let mut max_off = 0.0_f64;
        let mut p_idx = 0usize;
        let mut q_idx = 1usize;
        for (r, row) in a.iter().enumerate() {
            for (c, &val) in row.iter().enumerate().skip(r + 1) {
                if val.abs() > max_off {
                    max_off = val.abs();
                    p_idx = r;
                    q_idx = c;
                }
            }
        }
        if max_off < 1e-11 {
            break;
        }
        let app = a[p_idx][p_idx];
        let aqq = a[q_idx][q_idx];
        let apq = a[p_idx][q_idx];
        let tau = (aqq - app) / (2.0 * apq);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let cs = 1.0 / (1.0 + t * t).sqrt();
        let sn = t * cs;

        a[p_idx][p_idx] = app - t * apq;
        a[q_idx][q_idx] = aqq + t * apq;
        a[p_idx][q_idx] = 0.0;
        a[q_idx][p_idx] = 0.0;

        for k in 0..3 {
            if k != p_idx && k != q_idx {
                let akp = a[k][p_idx];
                let akq = a[k][q_idx];
                a[k][p_idx] = cs * akp - sn * akq;
                a[p_idx][k] = a[k][p_idx];
                a[k][q_idx] = sn * akp + cs * akq;
                a[q_idx][k] = a[k][q_idx];
            }
            let vkp = v[k][p_idx];
            let vkq = v[k][q_idx];
            v[k][p_idx] = cs * vkp - sn * vkq;
            v[k][q_idx] = sn * vkp + cs * vkq;
        }
    }

    let mut order = [0usize, 1, 2];
    order.sort_by(|&i, &j| {
        a[j][j]
            .partial_cmp(&a[i][i])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let col = |idx: usize| -> [f64; 3] { [v[0][idx], v[1][idx], v[2][idx]] };
    let mut e1 = col(order[0]); // 최대 분산 축 -> 수평 X (잔존폭)
    let mut e3 = col(order[2]); // 최소 분산 축 -> 수직 +Z (잔존고/두께)

    if e3[2] < -1e-6 || (e3[2].abs() <= 1e-6 && e3[1] < 0.0) {
        e3 = [-e3[0], -e3[1], -e3[2]];
    }
    if e1[0] < -1e-6 || (e1[0].abs() <= 1e-6 && e1[1] < 0.0) {
        e1 = [-e1[0], -e1[1], -e1[2]];
    }
    // 오른손 직교좌표계(det = +1): Y축(e2) = Z축(e3) × X축(e1)
    let mut e2 = [
        e3[1] * e1[2] - e3[2] * e1[1],
        e3[2] * e1[0] - e3[0] * e1[2],
        e3[0] * e1[1] - e3[1] * e1[0],
    ];
    let len2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2])
        .sqrt()
        .max(1e-12);
    e2 = [e2[0] / len2, e2[1] / len2, e2[2] / len2];

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;
    for p in &mesh.positions {
        let rx = e1[0] * p[0] + e1[1] * p[1] + e1[2] * p[2];
        let ry = e2[0] * p[0] + e2[1] * p[1] + e2[2] * p[2];
        let rz = e3[0] * p[0] + e3[1] * p[1] + e3[2] * p[2];
        min_x = min_x.min(rx);
        max_x = max_x.max(rx);
        min_y = min_y.min(ry);
        max_y = max_y.max(ry);
        min_z = min_z.min(rz);
    }
    let tx = -0.5 * (min_x + max_x);
    let ty = -0.5 * (min_y + max_y);
    let tz = -min_z;

    let mat = Mat4 {
        m: [
            e1[0], e1[1], e1[2], tx, e2[0], e2[1], e2[2], ty, e3[0], e3[1], e3[2], tz, 0.0, 0.0,
            0.0, 1.0,
        ],
    };
    mesh.transformed(&mat)
}
