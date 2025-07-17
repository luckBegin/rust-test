// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use scap::{
    capturer::{Point, Area, Size, Capturer, Options},
    frame::Frame,
};

fn main() {
    screen_lib::run();
}
