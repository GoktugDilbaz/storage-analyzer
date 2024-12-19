// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod disk_utils;
mod size_analysis;

fn main() {
    storage_analyzer_lib::run();
}
