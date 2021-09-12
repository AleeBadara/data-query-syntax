use regex::Regex;

pub fn verify_load_input(arg: &str) {
    let regex_load = Regex::new(r"load\(.+\..+\)\.separator\(.{1}\)").unwrap();
    match regex_load.find(&arg) {
        Some(_) => (),
        None => panic!("Invalid query. See how to use the load method in the documentation."),
    }
}

pub fn verify_select_input(arg: &str) {
    let regex = Regex::new(r"select\(\)").unwrap();
    match regex.find(&arg) {
        Some(_) => (),
        None => panic!("Invalid query. See how to use the select method in the documentation."),
    }
}
