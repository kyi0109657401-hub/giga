pub mod feature_lines;
pub mod hemp_fft;
pub mod rubbing_engine;
pub mod unroll;

pub use feature_lines::{extract_five_feature_lines, FeatureLineKind, FeatureSegment2D};
pub use hemp_fft::{measure_hemp_cloth_threads_per_cm, HempThreadCountResult};
pub use rubbing_engine::{
    compute_vertex_rubbing_ink_weights, render_orthographic_rubbing_image,
    suppress_isolated_ink_speckles, RubbingAlgorithm, RubbingRenderParams,
};
pub use unroll::{unroll_mesh_surface, UnrollMode, UnrollResult, UnrolledPoint2D};

#[cfg(test)]
mod tests {
    use super::*;
    use strata_core::{StacklessLbvh, TriMesh};

    #[test]
    fn test_rubbing_3_algorithms_and_5_feature_lines() {
        let positions = vec![
            [-20.0, 0.0, 0.0],
            [0.0, -5.0, 0.0],
            [20.0, 0.0, 0.0],
            [-20.0, 0.0, 30.0],
            [0.0, -8.0, 30.0],
            [20.0, 0.0, 30.0],
        ];
        let triangles = vec![[0, 1, 4], [0, 4, 3], [1, 2, 5], [1, 5, 4]];
        let mesh = TriMesh::from_positions_and_triangles("relief_patch", positions, triangles)
            .expect("ok");
        let bvh = StacklessLbvh::build(&mesh);

        for alg in [
            RubbingAlgorithm::NormalGradientHandprint,
            RubbingAlgorithm::Msii16Scale,
            RubbingAlgorithm::TopologicalOpenness,
        ] {
            let params = RubbingRenderParams {
                algorithm: alg,
                ink_density_pct: 75.0,
                curv_radius_mm: 1.5,
                use_handprint_grain: true,
            };
            let w = compute_vertex_rubbing_ink_weights(&mesh, Some(&bvh), params, [0.0, -1.0, 0.0]);
            assert_eq!(w.len(), mesh.positions.len());
            assert!(w.iter().all(|&v| (0.0..=1.0).contains(&v)));
        }

        let img = render_orthographic_rubbing_image(
            &mesh,
            Some(&bvh),
            RubbingRenderParams::default(),
            128,
            128,
        );
        assert_eq!(img.width(), 128);
        assert_eq!(img.height(), 128);

        let flines = extract_five_feature_lines(&mesh, [0.0, -1.0, 0.0]);
        assert!(!flines.is_empty());
    }

    #[test]
    fn test_cylindrical_unroll_and_hemp_fft_thread_count() {
        // Test 1: 1x1 cm hemp cloth grid with 8 warp threads x 10 weft threads per cm
        let n = 64usize;
        let mut grid = vec![0.0f64; n * n];
        for y in 0..n {
            for x in 0..n {
                let wx = (std::f64::consts::TAU * 8.0 * (x as f64) / (n as f64)).cos();
                let wy = (std::f64::consts::TAU * 10.0 * (y as f64) / (n as f64)).cos();
                grid[y * n + x] = wx + wy;
            }
        }
        let hemp = measure_hemp_cloth_threads_per_cm(&grid, n);
        assert_eq!(hemp.warp_threads_per_cm, 8);
        assert_eq!(hemp.weft_threads_per_cm, 10);
        assert!(hemp.format_kr().contains("8×10올"));

        // Test 2: Cylindrical Unroll
        let positions = vec![
            [30.0, 0.0, 0.0],
            [0.0, 30.0, 0.0],
            [-30.0, 0.0, 20.0],
            [0.0, -30.0, 20.0],
        ];
        let triangles = vec![[0, 1, 2], [0, 2, 3]];
        let mesh = TriMesh::from_positions_and_triangles("cyl", positions, triangles).expect("ok");
        let unrolled = unroll_mesh_surface(&mesh, UnrollMode::Cylindrical);
        assert!((unrolled.mean_radius_mm - 30.0).abs() < 1e-6);
        assert_eq!(unrolled.points.len(), 4);
    }

    #[test]
    fn test_orthographic_rubbing_uses_barycentric_and_zbuffer() {
        // ④: bbox 통칠 대신 barycentric 보간 + Z버퍼.
        // 단일 직각삼각형의 bbox 귀퉁이는 삼각형 밖이므로 바탕색 그대로여야 한다
        // (통칠이면 먹색으로 칠해진다).
        let positions = vec![[0.0, 0.0, 0.0], [40.0, 0.0, 0.0], [0.0, 40.0, 5.0]];
        let triangles = vec![[0, 1, 2]];
        let mesh =
            TriMesh::from_positions_and_triangles("single_tri", positions, triangles).expect("ok");
        let img = render_orthographic_rubbing_image(
            &mesh,
            None,
            RubbingRenderParams::default(),
            200,
            200,
        );
        // 렌더 매핑과 동일한 식으로 화면 좌표를 찾는다 (x폭 40, z폭 5 → scale 4.6).
        let (w, h) = (200.0, 200.0);
        let scale = ((w - 16.0) / 40.0_f64).min((h - 16.0) / 5.0);
        let (cx, cy, mesh_cx, mesh_cz) = (w * 0.5, h * 0.5, 20.0, 2.5);
        let to_px = |wx: f64, wz: f64| {
            (
                (cx + (wx - mesh_cx) * scale).floor() as u32,
                (cy - (wz - mesh_cz) * scale).floor() as u32,
            )
        };
        // (40, 2.5) → 화면 (192, 100): bbox 안·삼각형 밖.
        let (ox, oy) = to_px(40.0, 2.5);
        let corner = img.get_pixel(ox.min(199), oy.min(199));
        assert_eq!(
            corner.0,
            [250, 249, 245, 0],
            "bbox 귀퉁이가 통칠되면 안 됨: {corner:?} at ({ox},{oy})"
        );
        // (10, 1) → 화면 (54, 106): 삼각형 내부 — 먹이 있어야 한다.
        let (ix, iy) = to_px(10.0, 1.0);
        let inside = img.get_pixel(ix.min(199), iy.min(199));
        assert_ne!(
            inside.0,
            [250, 249, 245, 0],
            "삼각형 내부는 칠해져야 함: {inside:?} at ({ix},{iy})"
        );
    }

    #[test]
    fn test_suppress_isolated_rubbing_speckles_preserves_continuous_strokes() {
        use image::{Rgba, RgbaImage};

        // 32x32 한지 배경(#FAF9F5) 위에:
        // 1) 고립 점묘 잡음(Isolated 1px speckles) 10개
        // 2) 연속된 1px 두께 가로 먹선(x = 4..=27, y = 16)
        let mut img = RgbaImage::from_pixel(32, 32, Rgba([248, 247, 243, 255]));
        let speckle_coords = [
            (3, 3),
            (8, 4),
            (15, 5),
            (22, 6),
            (28, 7),
            (4, 25),
            (10, 26),
            (17, 27),
            (23, 28),
            (29, 24),
        ];
        for &(sx, sy) in &speckle_coords {
            img.put_pixel(sx, sy, Rgba([28, 28, 26, 255]));
        }
        for x in 4..=27 {
            img.put_pixel(x, 16, Rgba([24, 24, 22, 255]));
        }

        suppress_isolated_ink_speckles(&mut img);

        // 모든 고립 점묘 잡음은 제거되어 배경 청결도 >= 98% 달성
        for &(sx, sy) in &speckle_coords {
            let p = img.get_pixel(sx, sy).0;
            assert!(
                p[0] >= 225,
                "고립 점묘 잡음 ({sx},{sy})이 제거되어야 함, 실제 픽셀={p:?}"
            );
        }

        // 연속된 선각 먹선(내부 구간 x = 5..=26, y = 16)은 100% 보존
        for x in 5..=26 {
            let p = img.get_pixel(x, 16).0;
            assert!(
                p[0] <= 40,
                "연속된 능선 먹선 ({x},16)은 보존되어야 함, 실제 픽셀={p:?}"
            );
        }
    }
}
