use colored::*; // lib pour afficher du texte en couleur dans le terminal (https://crates.io/crates/colored)

pub fn show_welcome_title() {
    println!("{}", "Welcome to DQS (Data Query Syntax).".green());
}
pub fn show_welcome_message() {
    println!();
    println!(
        "{}",
        ">Load your data file and enter queries to request it.".blue()
    );
    println!("{}", ">Enter h for help or q to quit.".blue());
}

pub fn show_programm_end_message() {
    println!("{}", "Program ended.".green());
}

pub fn show_load_success_message() {
    println!("{}", "Data loaded successfully.".green());
}

pub fn show_umbiguous_message() {
    println!(
        "{}",
        "More than one query find. You cannot mix queries".red()
    );
}

pub fn show_unknown_command_message() {
    println!("{}", "Unknown query. See the documentation.".red());
}
