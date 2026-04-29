// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use handless_app_lib::CliArgs;

fn main() {
    let cli_args = CliArgs::parse();

    handless_app_lib::run(cli_args)
}
