use opencv::{
    core::{Mat, Vector},
    imgcodecs, prelude::*,
    videoio,
    imgproc,
};

use rayon::prelude::*;
use std::net::TcpListener;
use std::io::Write;

fn bayer_dithering(frame: &Mat, bayer_matrix: &[u8; 16]) -> Mat {
    let mut dithered_frame = Mat::default();
    frame.copy_to(&mut dithered_frame).unwrap();

    let cols = dithered_frame.cols();

    let scaled_bayer_matrix: [u8; 16] = bayer_matrix.map(|v| (v as f32 / 16.0 * 255.0) as u8);

    let data = dithered_frame.data_bytes_mut().unwrap();

    data.par_chunks_mut(cols as usize)
        .enumerate()
        .for_each(|(i, row)| {
            for j in 0..cols {
                let pixel = row[j as usize];
                let index = ((i % 4) as usize) * 4 + ((j % 4) as usize);
                let bayer_value = scaled_bayer_matrix[index];
                row[j as usize] = if pixel > bayer_value { 255 } else { 0 };
            }
        });

    dithered_frame
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080");

    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to get video capture");
    let mut frame = Mat::default();
    let mut gray_frame = Mat::default();
    let mut buf = Vector::new();

    // 4x4 Bayer matrix
    let bayer_matrix: [u8; 16] = [
        0, 8, 2, 10,
        12, 4, 14, 6,
        3, 11, 1, 9,
        15, 7, 13, 5,
    ];

    loop {
        let (mut stream, _) = listener.accept().expect("Failed to accept connection");

        cam.read(&mut frame).expect("Failed to capture frame");
        buf.clear();
        let _ = imgcodecs::imencode(".jpg", &frame, &mut buf, &Vector::new());

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
        );

        stream.write_all(response.as_bytes()).unwrap();

        loop {
            cam.read(&mut frame).expect("Failed to capture frame");

            imgproc::cvt_color(&frame, &mut gray_frame, imgproc::COLOR_BGR2GRAY, 0).unwrap();

            let dithered_frame = bayer_dithering(&gray_frame, &bayer_matrix);

            buf.clear();
            let _ = imgcodecs::imencode(".jpg", &dithered_frame, &mut buf, &Vector::new());

            let image_data = format!(
                "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                buf.len()
            );

            stream.write_all(image_data.as_bytes()).unwrap();
            stream.write_all(buf.as_slice()).unwrap();
            stream.write_all(b"\r\n").unwrap();
            stream.flush().unwrap();
        }
    }
}