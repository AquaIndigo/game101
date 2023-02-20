extern crate opencv;

use std::borrow::BorrowMut;
use nalgebra::{Vector2, Vector3};
use opencv::{
    Result,
};
use opencv::core::{CV_8UC3, Mat, MatTrait, MatTraitConst, Point, Scalar, VecN};
use opencv::highgui::{EVENT_LBUTTONDOWN, imshow, MouseCallback, named_window, set_mouse_callback, wait_key, WINDOW_AUTOSIZE};
use opencv::imgproc::{circle, COLOR_RGB2BGR, cvt_color};


static mut CONTROL_POINTS: Vec<Vector2<i32>> = vec![];


fn mouse_handler(event: i32, x: i32, y: i32, _flags: i32) {
    if event == EVENT_LBUTTONDOWN && unsafe { CONTROL_POINTS.len() } < 4 {
        println!("Left button of the mouse is clicked - position ({}, {})", x, y);
        unsafe { CONTROL_POINTS.push(Vector2::new(x, y)) };
    }
}

fn naive_bezier(points: &Vec<Vector2<f64>>, win: &mut Mat) {
    let p0 = points[0];
    let p1 = points[1];
    let p2 = points[2];
    let p3 = points[3];
    for i in 0..1000 {
        let t = i as f64 / 1000.0;
        let point = (1.0 - t).powf(3.0) * p0 + 3.0 * t * (1.0 - t).powf(2.0) * p1
            + 3.0 * t.powf(2.0) * (1.0 - t) * p2 + t.powf(3.0) * p3;
        let color = win.at_2d_mut::<VecN<u8, 3>>(point.y as i32, point.x as i32).unwrap();
        color[2] = 255;
    }
}

fn recursive_bezier(control_points: &Vec<Vector2<f64>>, t: f64) -> Vector2<f64> {
    let mut buffer = control_points.clone();
    let iter = control_points.len();
    for i in 0..iter - 1 {
        for j in 0..iter - i - 1 {
            buffer[j] = (1.0 - t) * buffer[j] + t * buffer[j + 1];
        }
    }
    buffer[0]
}

fn bezier(points: &Vec<Vector2<f64>>, win: &mut Mat) {
    for i in 0..1000 {
        let t = i as f64 / 1000.0;
        let res = recursive_bezier(points, t);
        win.at_2d_mut::<VecN<u8, 3>>(res.y as i32, res.x as i32).unwrap()[1] = 255;
    }
}

fn main() {
    let window = Mat::new_rows_cols_with_default(700, 700, CV_8UC3, Scalar::default()).unwrap();
    let mut win = Mat::copy(&window).unwrap();

    cvt_color(&window, &mut win, COLOR_RGB2BGR, 0).expect("convert error!");
    named_window("Bezier Curve", WINDOW_AUTOSIZE).unwrap();
    set_mouse_callback("Bezier Curve", Some(Box::new(mouse_handler))).unwrap();

    let mut k = -1;
    while k != 27 {
        unsafe {
            for point in &CONTROL_POINTS {
                let p = Point::new(point.x, point.y);
                circle(&mut win, p, 3, Scalar::from((255.0, 255.0, 255.0)), 3, 0, 0).unwrap();
            }
        }
        if unsafe { CONTROL_POINTS.len() } == 4 {
            let control_points: Vec<Vector2<f64>> = unsafe {
                CONTROL_POINTS.iter().map(|v| Vector2::new(v.x as f64, v.y as f64)).collect()
            };
            naive_bezier(&control_points, &mut win);
            bezier(&control_points, &mut win);
            imshow("Bezier Curve", &win).unwrap();
            k = wait_key(0).unwrap();
            return;
        }
        imshow("Bezier Curve", &win).unwrap();
        k = wait_key(20).unwrap();
    }
}