#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hoshii_lib::run().expect("failed to run desktop app");
}