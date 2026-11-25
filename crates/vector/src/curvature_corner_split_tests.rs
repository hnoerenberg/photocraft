//! #12699: a double-click on an anchor of a finished Curvature path makes a corner that splits
//! the curve into two independent smooth runs: each run is solved as an open Curvature path of
//! its own, and the corner keeps the end handle of each run (a cusp). The runs are pinned to the
//! handle solver run on each half, as the issue prescribes until #12698 lands; the not-measured
//! cases (closed subpaths, an open end, a two-anchor path) keep today's behaviour.

use super::*;

/// The measured path of #12699: an 800 x 600 document, five smooth anchors A..E.
const PTS: [[f64; 2]; 5] = [[100.0, 300.0], [250.0, 150.0], [400.0, 300.0], [550.0, 150.0], [700.0, 300.0]];

/// The five-anchor path as the Curvature Pen draws it: smooth knots at the solved handles.
fn five() -> Path {
    let knots = curvature_knots(&PTS, &[false; 5], false)
        .into_iter()
        .map(|k| Knot::smooth(Point::new(k[0][0], k[0][1]), Point::new(k[1][0], k[1][1]), Point::new(k[2][0], k[2][1])))
        .collect();
    Path::new(vec![Subpath { closed: false, knots, op: Default::default() }])
}

/// A knot as the solver's rows: `[anchor, in, out]`.
fn row(k: &Knot) -> [[f64; 2]; 3] {
    [[k.anchor.x, k.anchor.y], [k.in_ctrl.x, k.in_ctrl.y], [k.out_ctrl.x, k.out_ctrl.y]]
}

#[test]
fn a_double_clicked_corner_splits_the_path_into_two_open_runs() {
    let mut p = five();
    toggle_curvature_point(&mut p, [0, 2]).unwrap();
    // The runs A-C and C-E, each solved as an open Curvature path of its own.
    let (left, right) = (curvature_knots(&PTS[..3], &[false; 3], false), curvature_knots(&PTS[2..], &[false; 3], false));
    let k = &p.subpaths[0].knots[2];
    assert!(!k.smooth, "the double-click made C a corner");
    assert_eq!(row(k), [PTS[2], left[2][1], right[0][2]], "C is a cusp: it keeps the end handle of each run");
    let solved = [left[0], left[1], right[1], right[2]];
    for (i, (k, w)) in p.subpaths[0].knots.iter().filter(|k| k.smooth).zip(solved).enumerate() {
        assert_eq!(row(k), w, "smooth knot {i} sits in its own run");
    }
}

#[test]
fn dragging_an_anchor_within_a_run_moves_the_cusp_handles_of_the_corner() {
    let mut p = five();
    toggle_curvature_point(&mut p, [0, 2]).unwrap();
    move_anchors_curvature(&mut p, &[[0, 1]], [0.0, -40.0]).unwrap();
    let pts = [[100.0, 300.0], [250.0, 110.0], [400.0, 300.0], [550.0, 150.0], [700.0, 300.0]];
    let (left, right) = (curvature_knots(&pts[..3], &[false; 3], false), curvature_knots(&pts[2..], &[false; 3], false));
    let k = &p.subpaths[0].knots[2];
    assert_eq!(row(k), [pts[2], left[2][1], right[0][2]], "the cusp handles follow the dragged run");
    for (i, (k, w)) in p.subpaths[0].knots.iter().filter(|k| k.smooth).zip([left[0], left[1], right[1], right[2]]).enumerate() {
        assert_eq!(row(k), w, "smooth knot {i} sits in its own run");
    }
}

#[test]
fn double_clicking_the_corner_again_gives_the_smooth_five_point_handles() {
    let mut p = five();
    toggle_curvature_point(&mut p, [0, 2]).unwrap();
    toggle_curvature_point(&mut p, [0, 2]).unwrap();
    assert!(p.subpaths[0].knots[2].smooth, "C turned smooth again");
    let want = curvature_knots(&PTS, &[false; 5], false);
    for (k, w) in p.subpaths[0].knots.iter().zip(want) {
        assert_eq!(row(k), w, "the whole path is one run again");
    }
}

#[test]
fn a_corner_at_an_open_end_keeps_today_s_behaviour() {
    let mut p = five();
    toggle_curvature_point(&mut p, [0, 0]).unwrap();
    let k = &p.subpaths[0].knots[0];
    assert!(!k.smooth);
    assert_eq!(row(k), [PTS[0]; 3], "an end corner keeps its handles retracted, as today");
    let want = curvature_knots(&PTS, &[true, false, false, false, false], false);
    for (k, w) in p.subpaths[0].knots[1..].iter().zip(&want[1..]) {
        assert_eq!(row(k), *w, "the rest of the path keeps today's solve");
    }
}

#[test]
fn a_two_anchor_path_keeps_today_s_behaviour() {
    let pts = [[100.0, 300.0], [400.0, 300.0]];
    let two = || {
        Path::new(vec![Subpath {
            closed: false,
            knots: pts.iter().map(|a| Knot::smooth(Point::new(a[0], a[1]), Point::new(a[0], a[1]), Point::new(a[0], a[1]))).collect(),
            op: Default::default(),
        }])
    };
    let mut p = two();
    toggle_curvature_point(&mut p, [0, 1]).unwrap();
    assert!(!p.subpaths[0].knots[1].smooth);
    assert_eq!(row(&p.subpaths[0].knots[1]), [pts[1]; 3], "a two-anchor path solves no handles, as today");
    toggle_curvature_point(&mut p, [0, 1]).unwrap();
    assert_eq!(p, two());
}

#[test]
fn a_corner_on_a_closed_subpath_keeps_today_s_behaviour() {
    let square = || Path::new(vec![Subpath::polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)])]);
    let mut p = square();
    toggle_curvature_point(&mut p, [0, 1]).unwrap();
    let k = p.subpaths[0].knots[1];
    assert!(k.smooth && k.in_ctrl != k.anchor && k.out_ctrl != k.anchor, "the toggled knot follows its neighbours, as today: {k:?}");
    toggle_curvature_point(&mut p, [0, 1]).unwrap();
    assert_eq!(p, square(), "a new corner on a closed path retracts, as today");
}
