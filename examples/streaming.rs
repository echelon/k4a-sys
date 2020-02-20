#![allow(non_upper_case_globals)]
extern crate k4a_sys;
use std::ptr;
use std::env;
use k4a_sys::*;

fn open_device() -> Option<k4a_device_t> {
    unsafe {
        let device_count = k4a_sys::k4a_device_get_installed_count();
        if device_count == 0 {
            println!("No K4A devices found");
            return None;
        }
        let mut device: k4a_device_t = ptr::null_mut();
        if k4a_device_open(0, &mut device) != k4a_result_t::K4A_RESULT_SUCCEEDED {
            println!("Failed to open device");
            return None;
        }
        Some(device)
    }
}

fn main() {
    let timeout_in_ms = 1000;

    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("usage: {} FRAMECOUNT", args[0]);
        println!("Capture FRAMECOUNT color and depth frames from the device using the separate get frame APIs");
        panic!("Missing FRAMECOUNT argument");
    }

    let capture_frame_count: i32 = args[1].parse().expect("Unable to parse frame count as a number");
    println!("Capturing {} frames", capture_frame_count);

    let config = k4a_device_configuration_t {
        color_format : k4a_image_format_t::K4A_IMAGE_FORMAT_COLOR_MJPG,
        color_resolution : k4a_color_resolution_t::K4A_COLOR_RESOLUTION_2160P,
        depth_mode : k4a_depth_mode_t::K4A_DEPTH_MODE_NFOV_UNBINNED,
        camera_fps : k4a_fps_t::K4A_FRAMES_PER_SECOND_30,
        synchronized_images_only: false,
        depth_delay_off_color_usec: 0,
        wired_sync_mode: k4a_wired_sync_mode_t::K4A_WIRED_SYNC_MODE_STANDALONE,
        subordinate_delay_off_master_usec: 0,
        disable_streaming_indicator: false,
    };
    unsafe {
    let device: k4a_device_t = open_device().expect("Unable to open device");

    if k4a_device_start_cameras(device, &config) != k4a_result_t::K4A_RESULT_SUCCEEDED {
        panic!("Failed to start device");
    }

    let mut capture: k4a_capture_t = ptr::null_mut();
    for _ in 0..capture_frame_count {
        match k4a_device_get_capture(device, &mut capture, timeout_in_ms) {
            k4a_wait_result_t::K4A_WAIT_RESULT_SUCCEEDED => {}
            k4a_wait_result_t::K4A_WAIT_RESULT_TIMEOUT => {
                println!("Timed out waiting for a capture");
                continue;
            }
            k4a_wait_result_t::K4A_WAIT_RESULT_FAILED => {
                panic!("Failed to read a capture");
            }
        }
        println!("Capture");

        // Probe for a color image
        let mut image = k4a_capture_get_color_image(capture);
        if !image.is_null() {
            print!(" | Color res:{:4}x{:4} stride:{:5} ",
                    k4a_image_get_height_pixels(image),
                    k4a_image_get_width_pixels(image),
                    k4a_image_get_stride_bytes(image));
            k4a_image_release(image);
        } else {
            print!(" | Color None                       ");
        }

        // probe for a IR16 image
        image = k4a_capture_get_ir_image(capture);
        if !image.is_null() {
            print!(" | Ir16 res:{:4}x{:4} stride:{:5} ",
                    k4a_image_get_height_pixels(image),
                    k4a_image_get_width_pixels(image),
                    k4a_image_get_stride_bytes(image));
            k4a_image_release(image);
        } else {
            print!(" | Ir16 None                       ");
        }

        // Probe for a depth16 image
        image = k4a_capture_get_depth_image(capture);
        if !image.is_null() {
            println!(" | Depth16 res:{:4}x{:4} stride:{:5}",
                    k4a_image_get_height_pixels(image),
                    k4a_image_get_width_pixels(image),
                    k4a_image_get_stride_bytes(image));
            k4a_image_release(image);
        }
        else {
            println!(" | Depth16 None");
        }
        // fflush(stdout);
    }
    k4a_device_close(device);
    }
}
