// Copyright (c) Christopher Whitley (AristurtleDev). All rights reserved.
// Licensed under the MIT license.
// See LICENSE file in the project root for full license information.

// Pure geometry helpers used by the sprite editor.
//
// All functions operate on plain `[f32; 2]` arrays so they can be called
// from unit tests without an egui context.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RectHandle {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CirclePart {
    Interior,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CircleHandle {
    North,
    East,
    South,
    West,
}

/// Returns the closest point on segment `a`–`b` to point `p`.
///
/// When `a == b` (degenerate segment), returns `a`.
pub(crate) fn closest_point_on_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ab_len_sq = ab[0] * ab[0] + ab[1] * ab[1];
    if ab_len_sq == 0.0 {
        // Degenerate segment; both endpoints are the same point.
        return a;
    }
    let ap = [p[0] - a[0], p[1] - a[1]];
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / ab_len_sq).clamp(0.0, 1.0);
    [a[0] + t * ab[0], a[1] + t * ab[1]]
}

/// Tests whether point `(x, y)` is inside the closed polygon defined by `pts`.
pub(crate) fn point_in_polygon(x: f32, y: f32, pts: &[[f32; 2]]) -> bool {
    let n = pts.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let [xi, yi] = pts[i];
        let [xj, yj] = pts[j];
        // Cast a ray in the +x direction and count crossings.
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Returns the `RectHandle` whose position is within `threshold` pixels of
/// `(px, py)`, or `None` if no handle is close enough.
pub(crate) fn rect_handle_hit_test(rect_x: f32, rect_y: f32, rect_w: f32, rect_h: f32, px: f32, py: f32, threshold: f32) -> Option<RectHandle> {
    let cx = rect_x + rect_w * 0.5;
    let cy = rect_y + rect_h * 0.5;
    let right = rect_x + rect_w;
    let bottom = rect_y + rect_h;
    let t_sq = threshold * threshold;

    let handles: &[([f32; 2], RectHandle)] = &[
        ([rect_x, rect_y], RectHandle::TopLeft),
        ([cx, rect_y], RectHandle::TopCenter),
        ([right, rect_y], RectHandle::TopRight),
        ([rect_x, cy], RectHandle::MiddleLeft),
        ([right, cy], RectHandle::MiddleRight),
        ([rect_x, bottom], RectHandle::BottomLeft),
        ([cx, bottom], RectHandle::BottomCenter),
        ([right, bottom], RectHandle::BottomRight),
    ];

    handles.iter().find_map(|&([hx, hy], handle)| {
        let dx = px - hx;
        let dy = py - hy;
        if dx * dx + dy * dy <= t_sq { Some(handle) } else { None }
    })
}

/// Returns which part of the circle was hit, or `None` if the point misses.
pub(crate) fn circle_hit_test(cx: f32, cy: f32, radius: f32, px: f32, py: f32, edge_threshold: f32) -> Option<CirclePart> {
    let dx = px - cx;
    let dy = py - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    let delta = (dist - radius).abs();
    if delta <= edge_threshold {
        Some(CirclePart::Edge)
    } else if dist < radius.max(0.0) - edge_threshold {
        Some(CirclePart::Interior)
    } else {
        None
    }
}

/// Returns the `CircleHandle` whose position is within `threshold` pixels of
/// `(px, py)`, or `None` if no handle is close enough.
pub(crate) fn circle_handle_hit_test(cx: f32, cy: f32, radius: f32, px: f32, py: f32, threshold: f32) -> Option<CircleHandle> {
    let t_sq = threshold * threshold;
    let handles: &[([f32; 2], CircleHandle)] = &[
        ([cx, cy - radius], CircleHandle::North),
        ([cx + radius, cy], CircleHandle::East),
        ([cx, cy + radius], CircleHandle::South),
        ([cx - radius, cy], CircleHandle::West),
    ];
    handles.iter().find_map(|&([hx, hy], handle)| {
        let dx = px - hx;
        let dy = py - hy;
        if dx * dx + dy * dy <= t_sq { Some(handle) } else { None }
    })
}

/// Returns the index of the first polygon vertex within `threshold` pixels of
/// `(px, py)`, or `None` if no vertex is close enough.
pub(crate) fn polygon_vertex_hit_test(pts: &[[f32; 2]], px: f32, py: f32, threshold: f32) -> Option<usize> {
    let t_sq = threshold * threshold;
    pts.iter().enumerate().find_map(|(i, &[vx, vy])| {
        let dx = px - vx;
        let dy = py - vy;
        if dx * dx + dy * dy <= t_sq { Some(i) } else { None }
    })
}

/// Clamps a rectangle's position and size so it stays within
/// `[0, bounds_w] x [0, bounds_h]` in sprite-local pixel coordinates.
pub(crate) fn clamp_rect_to_bounds(x: f32, y: f32, w: f32, h: f32, bounds_w: f32, bounds_h: f32) -> (f32, f32, f32, f32) {
    let cw = w.clamp(1.0, bounds_w);
    let ch = h.clamp(1.0, bounds_h);
    let cx = x.clamp(0.0, (bounds_w - cw).max(0.0));
    let cy = y.clamp(0.0, (bounds_h - ch).max(0.0));
    (cx, cy, cw, ch)
}

pub(crate) fn clamp_circle_to_bounds(cx: f32, cy: f32, radius: f32, bounds_w: f32, bounds_h: f32) -> (f32, f32, f32) {
    let max_r = (bounds_w.min(bounds_h) / 2.0).max(1.0);
    let r = radius.clamp(1.0, max_r);

    let ncx = cx.clamp(r, (bounds_w - r).max(r));
    let ncy = cy.clamp(r, (bounds_h - r).max(r));
    (ncx, ncy, r)
}

/// Clamps each polygon vertex individually to `[0, bounds_w] x [0, bounds_h]`
/// in sprite-local pixel coordinates.
pub(crate) fn clamp_polygon_to_bounds(points: &[[f32; 2]], bounds_w: f32, bounds_h: f32) -> Vec<[f32; 2]> {
    points.iter().map(|&[x, y]| [x.clamp(0.0, bounds_w), y.clamp(0.0, bounds_h)]).collect()
}

/// Returns the largest `(dx, dy)` (preserving sign) such that translating every
/// vertex by the result keeps all of them within `[0, bounds_w] x [0, bounds_h]`.
pub(crate) fn clamp_polygon_translation(points: &[[f32; 2]], dx: f32, dy: f32, bounds_w: f32, bounds_h: f32) -> (f32, f32) {
    if points.is_empty() {
        return (dx, dy);
    }
    // Each vertex [px, py] constrains dx to [-px, bounds_w - px] and
    // dy to [-py, bounds_h - py]. Intersect across all vertices.
    let mut dx_min = f32::NEG_INFINITY;
    let mut dx_max = f32::INFINITY;
    let mut dy_min = f32::NEG_INFINITY;
    let mut dy_max = f32::INFINITY;
    for &[px, py] in points {
        dx_min = dx_min.max(-px);
        dx_max = dx_max.min(bounds_w - px);
        dy_min = dy_min.max(-py);
        dy_max = dy_max.min(bounds_h - py);
    }
    (dx.clamp(dx_min, dx_max), dy.clamp(dy_min, dy_max))
}

/// Triangulates a simple polygon using ear-clipping algorithm.
pub(crate) fn ear_clip_triangulate(pts: &[[f32; 2]]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[0, 1, 2]];
    }
    // Signed area (shoelace formula). In screen coordinates (Y-down):
    // positive -> clockwise, negative -> counter-clockwise.
    let signed_area: f32 = (0..n)
        .map(|i| {
            let [x0, y0] = pts[i];
            let [x1, y1] = pts[(i + 1) % n];
            x0 * y1 - x1 * y0
        })
        .sum::<f32>()
        * 0.5;

    let mut indices: Vec<usize> = (0..n).collect();
    let mut triangles: Vec<[usize; 3]> = Vec::with_capacity(n - 2);

    while indices.len() > 3 {
        let m = indices.len();
        let mut ear_found = false;
        for i in 0..m {
            let prev = indices[(i + m - 1) % m];
            let curr = indices[i];
            let next = indices[(i + 1) % m];
            let a = pts[prev];
            let b = pts[curr];
            let c = pts[next];

            // Cross product z-component of (b − a) x (c − a).
            // For a convex (ear-candidate) vertex its sign must match
            // the polygon's winding direction.
            let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            let is_convex = (signed_area > 0.0 && cross > 0.0) || (signed_area < 0.0 && cross < 0.0);
            if !is_convex {
                continue;
            }

            // An ear is invalid if any other polygon vertex lies inside the triangle.
            let is_ear = indices.iter().all(|&idx| {
                if idx == prev || idx == curr || idx == next {
                    return true;
                }
                !point_in_triangle(pts[idx], a, b, c)
            });

            if is_ear {
                triangles.push([prev, curr, next]);
                indices.remove(i);
                ear_found = true;
                break;
            }
        }
        if !ear_found {
            // Degenerate polygon (e.g. collinear points); cannot continue.
            break;
        }
    }

    if indices.len() == 3 {
        triangles.push([indices[0], indices[1], indices[2]]);
    }

    triangles
}

/// Returns `true` if `p` lies strictly inside or on the boundary of triangle
/// `(a, b, c)`. Works for any vertex winding order.
fn point_in_triangle(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let d1 = edge_sign(p, a, b);
    let d2 = edge_sign(p, b, c);
    let d3 = edge_sign(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn edge_sign(p1: [f32; 2], p2: [f32; 2], p3: [f32; 2]) -> f32 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}

/// Returns `(edge_start_index, closest_point)` for the polygon edge closest to
/// `(px, py)` within `threshold` pixels, or `None` if no edge qualifies.
pub(crate) fn polygon_closest_edge_point(pts: &[[f32; 2]], px: f32, py: f32, threshold: f32) -> Option<(usize, [f32; 2])> {
    if pts.len() < 2 {
        return None;
    }
    let t_sq = threshold * threshold;
    (0..pts.len())
        .filter_map(|i| {
            let c = closest_point_on_segment([px, py], pts[i], pts[(i + 1) % pts.len()]);
            let d_sq = (px - c[0]).powi(2) + (py - c[1]).powi(2);
            (d_sq <= t_sq).then_some((i, c, d_sq))
        })
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, c, _)| (i, c))
}

/// Returns `true` if the polygon defined by `pts` is self-intersecting, i.e. any
/// two non-adjacent edges cross each other.
pub(crate) fn polygon_is_self_intersecting(pts: &[[f32; 2]]) -> bool {
    let n = pts.len();
    if n < 4 {
        return false;
    }
    for i in 0..n {
        let a1 = pts[i];
        let a2 = pts[(i + 1) % n];
        // j starts at i+2 to skip the immediately adjacent edge.
        // The pair (i=0, j=n-1) is also adjacent (they share vertex 0); the
        // proper-intersection test naturally returns false for that case because
        // b2 == a1, making one of the signed-area values zero.
        for j in (i + 2)..n {
            let b1 = pts[j];
            let b2 = pts[(j + 1) % n];
            if segments_properly_intersect(a1, a2, b1, b2) {
                return true;
            }
        }
    }
    false
}

/// Returns `true` if adding `new_vertex` to the open polyline `pts` would create
/// a self-intersecting edge.
pub(crate) fn polyline_would_self_intersect_adding_vertex(pts: &[[f32; 2]], new_vertex: [f32; 2]) -> bool {
    let n = pts.len();
    if n < 2 {
        return false;
    }
    let new_start = pts[n - 1];
    // Non-adjacent existing edges: indices 0 .. (n-2) exclusive, i.e. 0 .. n.saturating_sub(2).
    for i in 0..n.saturating_sub(2) {
        if segments_properly_intersect(new_start, new_vertex, pts[i], pts[i + 1]) {
            return true;
        }
    }
    false
}

/// Returns `true` if segments `a1–a2` and `b1–b2` properly cross each other
/// (both endpoints of each segment lie on strictly opposite sides of the other
/// segment's supporting line). Collinear or endpoint-touching cases return `false`.
fn segments_properly_intersect(a1: [f32; 2], a2: [f32; 2], b1: [f32; 2], b2: [f32; 2]) -> bool {
    // orient(p, q, r) = signed area of triangle (p, q, r).
    // Positive -> left turn (CCW), negative -> right turn (CW), zero -> collinear.
    let orient = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| -> f32 { (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]) };
    let d1 = orient(a1, a2, b1);
    let d2 = orient(a1, a2, b2);
    let d3 = orient(b1, b2, a1);
    let d4 = orient(b1, b2, a2);
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_point_on_segment_degenerate() {
        // When a == b the result must equal a regardless of p.
        let result = closest_point_on_segment([5.0, 7.0], [1.0, 2.0], [1.0, 2.0]);
        assert_eq!(result, [1.0, 2.0]);
    }

    #[test]
    fn closest_point_on_segment_perpendicular_foot_inside() {
        // Foot of perpendicular from (2, 3) to the segment (0,0)-(4,0) is (2,0).
        let result = closest_point_on_segment([2.0, 3.0], [0.0, 0.0], [4.0, 0.0]);
        assert!((result[0] - 2.0).abs() < 1e-5, "x should be 2.0, got {}", result[0]);
        assert!((result[1] - 0.0).abs() < 1e-5, "y should be 0.0, got {}", result[1]);
    }

    #[test]
    fn closest_point_on_segment_point_past_end() {
        // Point (10, 0) is beyond endpoint (4, 0); result should be clamped to (4, 0).
        let result = closest_point_on_segment([10.0, 0.0], [0.0, 0.0], [4.0, 0.0]);
        assert!((result[0] - 4.0).abs() < 1e-5);
        assert!((result[1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn closest_point_on_segment_point_before_start() {
        // Point (-5, 0) is before start (0, 0); result should be clamped to (0, 0).
        let result = closest_point_on_segment([-5.0, 0.0], [0.0, 0.0], [4.0, 0.0]);
        assert!((result[0] - 0.0).abs() < 1e-5);
        assert!((result[1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn point_in_polygon_two_vertices_returns_false() {
        assert!(!point_in_polygon(0.5, 0.5, &[[0.0, 0.0], [1.0, 1.0]]));
    }

    #[test]
    fn point_in_polygon_inside_unit_square() {
        let square = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(point_in_polygon(0.5, 0.5, &square));
    }

    #[test]
    fn point_in_polygon_outside_unit_square() {
        let square = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(!point_in_polygon(2.0, 2.0, &square));
        assert!(!point_in_polygon(-0.1, 0.5, &square));
    }

    #[test]
    fn point_in_polygon_nonconvex_l_shape_inside() {
        // L-shaped polygon (6 vertices).
        //  (0,0)---(2,0)
        //           |
        //          (2,1)--(3,1)
        //                  |
        //  (0,2)----------(3,2)
        let l_shape = [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [3.0, 1.0], [3.0, 2.0], [0.0, 2.0]];
        // Point inside the top-left arm of the L.
        assert!(point_in_polygon(1.0, 0.5, &l_shape));
        // Point inside the bottom-right arm.
        assert!(point_in_polygon(2.5, 1.5, &l_shape));
    }

    #[test]
    fn point_in_polygon_nonconvex_l_shape_outside() {
        let l_shape = [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [3.0, 1.0], [3.0, 2.0], [0.0, 2.0]];
        // Point in the concave notch (top-right of the L) is outside.
        assert!(!point_in_polygon(2.5, 0.5, &l_shape));
    }

    #[test]
    fn rect_handle_hit_test_top_left_corner() {
        // Exact corner position should hit TopLeft.
        let hit = rect_handle_hit_test(0.0, 0.0, 100.0, 50.0, 0.0, 0.0, 5.0);
        assert_eq!(hit, Some(RectHandle::TopLeft));
    }

    #[test]
    fn rect_handle_hit_test_outside_all_handles() {
        // Far from the rectangle; should miss.
        let hit = rect_handle_hit_test(0.0, 0.0, 100.0, 50.0, 200.0, 200.0, 5.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn rect_handle_hit_test_bottom_right_corner() {
        let hit = rect_handle_hit_test(0.0, 0.0, 100.0, 50.0, 100.0, 50.0, 5.0);
        assert_eq!(hit, Some(RectHandle::BottomRight));
    }

    #[test]
    fn rect_handle_hit_test_top_center_edge() {
        // Midpoint of top edge.
        let hit = rect_handle_hit_test(0.0, 0.0, 100.0, 50.0, 50.0, 0.0, 5.0);
        assert_eq!(hit, Some(RectHandle::TopCenter));
    }

    #[test]
    fn circle_hit_test_interior() {
        // Point at center is well inside.
        let hit = circle_hit_test(50.0, 50.0, 30.0, 50.0, 50.0, 4.0);
        assert_eq!(hit, Some(CirclePart::Interior));
    }

    #[test]
    fn circle_hit_test_edge() {
        // Point exactly on the boundary is on the edge.
        let hit = circle_hit_test(0.0, 0.0, 30.0, 30.0, 0.0, 4.0);
        assert_eq!(hit, Some(CirclePart::Edge));
    }

    #[test]
    fn circle_hit_test_outside() {
        // Point far outside the circle.
        let hit = circle_hit_test(0.0, 0.0, 30.0, 100.0, 0.0, 4.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn polygon_vertex_hit_test_hits_first_vertex() {
        let pts = [[0.0_f32, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let hit = polygon_vertex_hit_test(&pts, 1.0, 0.0, 5.0);
        assert_eq!(hit, Some(0));
    }

    #[test]
    fn polygon_vertex_hit_test_hits_last_vertex() {
        let pts = [[0.0_f32, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let hit = polygon_vertex_hit_test(&pts, 5.5, 10.0, 5.0);
        assert_eq!(hit, Some(2));
    }

    #[test]
    fn polygon_vertex_hit_test_misses_all() {
        let pts = [[0.0_f32, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let hit = polygon_vertex_hit_test(&pts, 50.0, 50.0, 5.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn polygon_vertex_hit_test_empty_returns_none() {
        let hit = polygon_vertex_hit_test(&[], 0.0, 0.0, 5.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn circle_handle_hit_test_north() {
        // North handle is at (cx, cy - radius) = (50, 20).
        let hit = circle_handle_hit_test(50.0, 50.0, 30.0, 50.0, 20.0, 5.0);
        assert_eq!(hit, Some(CircleHandle::North));
    }

    #[test]
    fn circle_handle_hit_test_east() {
        // East handle is at (cx + radius, cy) = (80, 50).
        let hit = circle_handle_hit_test(50.0, 50.0, 30.0, 80.0, 50.0, 5.0);
        assert_eq!(hit, Some(CircleHandle::East));
    }

    #[test]
    fn circle_handle_hit_test_south() {
        // South handle is at (cx, cy + radius) = (50, 80).
        let hit = circle_handle_hit_test(50.0, 50.0, 30.0, 50.0, 80.0, 5.0);
        assert_eq!(hit, Some(CircleHandle::South));
    }

    #[test]
    fn circle_handle_hit_test_west() {
        // West handle is at (cx - radius, cy) = (20, 50).
        let hit = circle_handle_hit_test(50.0, 50.0, 30.0, 20.0, 50.0, 5.0);
        assert_eq!(hit, Some(CircleHandle::West));
    }

    #[test]
    fn circle_handle_hit_test_miss() {
        // Point at the circle centre is not near any cardinal handle.
        let hit = circle_handle_hit_test(50.0, 50.0, 30.0, 50.0, 50.0, 5.0);
        assert_eq!(hit, None);
    }

    #[test]
    fn clamp_rect_already_in_bounds() {
        // Rect fully inside bounds - all values unchanged.
        let (x, y, w, h) = clamp_rect_to_bounds(5.0, 10.0, 20.0, 15.0, 100.0, 100.0);
        assert_eq!((x, y, w, h), (5.0, 10.0, 20.0, 15.0));
    }

    #[test]
    fn clamp_rect_negative_origin() {
        // x < 0 should be clamped to 0; size unchanged.
        let (x, y, w, h) = clamp_rect_to_bounds(-5.0, 0.0, 20.0, 20.0, 100.0, 100.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 20.0);
        assert_eq!(h, 20.0);
    }

    #[test]
    fn clamp_rect_extends_past_right_edge() {
        // x=90, w=20 -> right edge at 110, exceeding bounds_w=100.
        let (x, _y, w, _h) = clamp_rect_to_bounds(90.0, 0.0, 20.0, 20.0, 100.0, 100.0);
        assert_eq!(w, 20.0); // w fits within bounds
        assert_eq!(x, 80.0); // x clamped so x+w == 100
    }

    #[test]
    fn clamp_rect_width_exceeds_bounds() {
        // w > bounds_w: clamp w, then clamp x.
        let (x, _y, w, _h) = clamp_rect_to_bounds(5.0, 0.0, 150.0, 20.0, 100.0, 100.0);
        assert_eq!(w, 100.0); // capped at bounds_w
        assert_eq!(x, 0.0); // x must be 0 when w == bounds_w
    }

    #[test]
    fn clamp_circle_already_in_bounds() {
        // Circle centered at (50,50) with radius 20 fits in 100x100.
        let (cx, cy, r) = clamp_circle_to_bounds(50.0, 50.0, 20.0, 100.0, 100.0);
        assert_eq!((cx, cy, r), (50.0, 50.0, 20.0));
    }

    #[test]
    fn clamp_circle_center_too_close_to_left() {
        // cx=5, radius=20 -> left edge at -15, outside bounds.
        let (cx, _cy, r) = clamp_circle_to_bounds(5.0, 50.0, 20.0, 100.0, 100.0);
        assert_eq!(r, 20.0);
        assert_eq!(cx, 20.0); // clamped to radius
    }

    #[test]
    fn clamp_circle_radius_too_large() {
        // radius=60 in 100x100 bounds: max_r = 50.
        let (cx, cy, r) = clamp_circle_to_bounds(50.0, 50.0, 60.0, 100.0, 100.0);
        assert_eq!(r, 50.0);
        // Center is (50,50); with r=50 it just fits: [50,50] is valid.
        assert_eq!(cx, 50.0);
        assert_eq!(cy, 50.0);
    }

    #[test]
    fn clamp_polygon_already_in_bounds() {
        let pts = [[10.0_f32, 10.0], [50.0, 10.0], [30.0, 40.0]];
        let clamped = clamp_polygon_to_bounds(&pts, 100.0, 100.0);
        assert_eq!(clamped, pts.to_vec());
    }

    #[test]
    fn clamp_polygon_vertex_out_of_bounds_right() {
        // One vertex beyond bounds_w; others unchanged.
        let pts = [[10.0_f32, 10.0], [120.0, 10.0], [30.0, 40.0]];
        let clamped = clamp_polygon_to_bounds(&pts, 100.0, 100.0);
        assert_eq!(clamped[0], [10.0, 10.0]);
        assert_eq!(clamped[1], [100.0, 10.0]); // x clamped to bounds_w
        assert_eq!(clamped[2], [30.0, 40.0]);
    }

    #[test]
    fn clamp_polygon_vertex_negative_y() {
        // Vertex above the sprite top (y < 0) clamped to 0.
        let pts = [[10.0_f32, -5.0], [50.0, 10.0], [30.0, 40.0]];
        let clamped = clamp_polygon_to_bounds(&pts, 100.0, 100.0);
        assert_eq!(clamped[0], [10.0, 0.0]);
        assert_eq!(clamped[1], [50.0, 10.0]);
        assert_eq!(clamped[2], [30.0, 40.0]);
    }

    #[test]
    fn clamp_polygon_translation_no_clip_needed() {
        // Triangle well inside 100x100 bounds; full delta passes through.
        let pts = [[10.0_f32, 10.0], [50.0, 10.0], [30.0, 40.0]];
        let (dx, dy) = clamp_polygon_translation(&pts, 5.0, 5.0, 100.0, 100.0);
        assert!((dx - 5.0).abs() < 1e-5 && (dy - 5.0).abs() < 1e-5);
    }

    #[test]
    fn clamp_polygon_translation_right_edge_stops_whole_body() {
        // Rightmost vertex at x=90 in 100-wide sprite; dx=20 should be clamped to 10.
        let pts = [[10.0_f32, 10.0], [90.0, 10.0], [50.0, 40.0]];
        let (dx, _dy) = clamp_polygon_translation(&pts, 20.0, 0.0, 100.0, 100.0);
        assert!((dx - 10.0).abs() < 1e-5, "expected dx=10.0, got {dx}");
    }

    #[test]
    fn clamp_polygon_translation_left_edge_stops_whole_body() {
        // Leftmost vertex at x=5; dragging left dx=-10 clamped to -5.
        let pts = [[5.0_f32, 20.0], [50.0, 20.0], [30.0, 60.0]];
        let (dx, _dy) = clamp_polygon_translation(&pts, -10.0, 0.0, 100.0, 100.0);
        assert!((dx - (-5.0)).abs() < 1e-5, "expected dx=-5.0, got {dx}");
    }

    #[test]
    fn clamp_polygon_translation_empty_polygon_passthrough() {
        let (dx, dy) = clamp_polygon_translation(&[], 7.0, -3.0, 100.0, 100.0);
        assert!((dx - 7.0).abs() < 1e-5 && (dy - (-3.0)).abs() < 1e-5);
    }

    #[test]
    fn ear_clip_triangle_returns_single_triplet() {
        let pts = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let tris = ear_clip_triangulate(&pts);
        assert_eq!(tris, vec![[0, 1, 2]]);
    }

    #[test]
    fn ear_clip_two_points_returns_empty() {
        assert!(ear_clip_triangulate(&[[0.0_f32, 0.0], [1.0, 1.0]]).is_empty());
    }

    fn triangle_area(pts: &[[f32; 2]], tri: [usize; 3]) -> f32 {
        let [ax, ay] = pts[tri[0]];
        let [bx, by] = pts[tri[1]];
        let [cx, cy] = pts[tri[2]];
        ((bx - ax) * (cy - ay) - (by - ay) * (cx - ax)).abs() * 0.5
    }

    #[test]
    fn ear_clip_convex_quad_covers_correct_area() {
        // CCW square in screen coords: area = 1.0, should produce 2 triangles.
        let pts = [[0.0_f32, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
        let tris = ear_clip_triangulate(&pts);
        assert_eq!(tris.len(), 2);
        let total: f32 = tris.iter().map(|&t| triangle_area(&pts, t)).sum();
        assert!((total - 1.0).abs() < 1e-4, "expected area 1.0, got {total}");
    }

    #[test]
    fn ear_clip_concave_l_shape_covers_correct_area() {
        // CW L-shape in screen coords; area = 3.0, needs 4 triangles.
        let pts = [[0.0_f32, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 2.0], [0.0, 2.0]];
        let tris = ear_clip_triangulate(&pts);
        assert_eq!(tris.len(), 4, "L-shape (6 verts) needs 4 triangles");
        let total: f32 = tris.iter().map(|&t| triangle_area(&pts, t)).sum();
        assert!((total - 3.0).abs() < 1e-4, "expected area 3.0, got {total}");
    }

    #[test]
    fn polygon_is_self_intersecting_triangle_returns_false() {
        let pts = [[0.0_f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        assert!(!polygon_is_self_intersecting(&pts));
    }

    #[test]
    fn polygon_is_self_intersecting_convex_square_returns_false() {
        let pts = [[0.0_f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(!polygon_is_self_intersecting(&pts));
    }

    #[test]
    fn polygon_is_self_intersecting_bowtie_returns_true() {
        // Bowtie: (0,0)->(2,0)->(0,2)->(2,2) - edges 1 and 3 cross at (1,1).
        let pts = [[0.0_f32, 0.0], [2.0, 0.0], [0.0, 2.0], [2.0, 2.0]];
        assert!(polygon_is_self_intersecting(&pts));
    }

    #[test]
    fn polygon_is_self_intersecting_concave_l_shape_returns_false() {
        // Concave (but simple) L-shape must not be flagged as self-intersecting.
        let pts = [[0.0_f32, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 2.0], [0.0, 2.0]];
        assert!(!polygon_is_self_intersecting(&pts));
    }

    #[test]
    fn polyline_no_intersection_empty_or_one_point() {
        // Fewer than 2 existing points - no existing edge can be crossed.
        assert!(!polyline_would_self_intersect_adding_vertex(&[], [5.0, 5.0]));
        assert!(!polyline_would_self_intersect_adding_vertex(&[[0.0, 0.0]], [5.0, 5.0]));
    }

    #[test]
    fn polyline_no_intersection_two_points() {
        // With exactly 2 existing points the only existing edge is adjacent to the new
        // edge (they share pts[1]), so no non-adjacent crossing is possible.
        let pts = [[0.0_f32, 0.0], [10.0, 0.0]];
        assert!(!polyline_would_self_intersect_adding_vertex(&pts, [5.0, 5.0]));
    }

    #[test]
    fn polyline_no_intersection_valid_square_completion() {
        // Adding (0,10) to [[0,0],[10,0],[10,10]] completes a square - no crossing.
        let pts = [[0.0_f32, 0.0], [10.0, 0.0], [10.0, 10.0]];
        assert!(!polyline_would_self_intersect_adding_vertex(&pts, [0.0, 10.0]));
    }

    #[test]
    fn polyline_detects_crossing_new_edge() {
        // pts = [[0,0],[10,10],[10,0]]; adding [0,10] creates edge [10,0]->[0,10]
        // which crosses the existing non-adjacent edge [0,0]->[10,10].
        let pts = [[0.0_f32, 0.0], [10.0, 10.0], [10.0, 0.0]];
        assert!(polyline_would_self_intersect_adding_vertex(&pts, [0.0, 10.0]));
    }

    #[test]
    fn polygon_closest_edge_point_hits_bottom_edge() {
        // Triangle bottom edge (0,0)->(10,0); point at (5,1) is 1 px above the edge.
        let pts = [[0.0_f32, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let (idx, pt) = polygon_closest_edge_point(&pts, 5.0, 1.0, 3.0).unwrap();
        assert_eq!(idx, 0);
        assert!((pt[0] - 5.0).abs() < 1e-5 && pt[1].abs() < 1e-5);
    }

    #[test]
    fn polygon_closest_edge_point_too_far_returns_none() {
        let pts = [[0.0_f32, 0.0], [10.0, 0.0], [5.0, 10.0]];
        assert!(polygon_closest_edge_point(&pts, 5.0, 20.0, 3.0).is_none());
    }
}
