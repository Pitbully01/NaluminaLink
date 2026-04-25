use std::error::Error;

mod i18n;
mod models;
mod node_discovery;
mod ui;

use i18n::I18n;
use node_discovery::{collect_nodes, render_nodes};
use ui::run_desktop_ui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let i18n = I18n::load();

    match args.get(1).map(String::as_str) {
        Some("ui") => run_ui_or_exit(&i18n),
        Some("list-nodes") => run_list_nodes_or_exit(&i18n),
        Some("doctor") => println!("{}", i18n.text("doctor.message")),
        Some("help") => print_help(&i18n),
        None => run_ui_or_exit(&i18n),
        Some(command) => {
            eprintln!(
                "{}",
                i18n.text_with("error.unknown_command", &[("command", command.to_string())])
            );
            print_help(&i18n);
            std::process::exit(1);
        }
    }
}

fn run_ui_or_exit(i18n: &I18n) {
    if let Err(error) = run_desktop_ui(i18n) {
        eprintln!(
            "{}",
            i18n.text_with("error.gui", &[("error", error.to_string())])
        );
        std::process::exit(1);
    }
}

fn run_list_nodes_or_exit(i18n: &I18n) {
    if let Err(error) = list_nodes(i18n) {
        eprintln!(
            "{}",
            i18n.text_with("error.list_nodes", &[("error", error.to_string())])
        );
        std::process::exit(1);
    }
}

fn print_help(i18n: &I18n) {
    println!("{}", i18n.text("cli.subtitle"));
    println!();
    println!("{}", i18n.text("cli.usage"));
    println!("{}", i18n.text("cli.help"));
    println!("{}", i18n.text("cli.doctor"));
    println!("{}", i18n.text("cli.ui"));
    println!("{}", i18n.text("cli.list_nodes"));
}

fn list_nodes(i18n: &I18n) -> Result<(), Box<dyn Error>> {
    let nodes = collect_nodes()?;

    if nodes.is_empty() {
        println!("{}", i18n.text("nodes.empty"));
        return Ok(());
    }

    render_nodes(i18n, &nodes);
    Ok(())
}
