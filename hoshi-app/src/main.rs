#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(mobile))]
fn main() {
    hoshii_lib::run().expect("failed to run desktop app");
}