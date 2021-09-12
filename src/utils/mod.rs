use crate::models;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn get_user_input() -> String {
    let mut user_input = String::new();
    io::stdin()
        .lock()
        .read_line(&mut user_input)
        .expect("Unable to read your input.");
    user_input
}
pub fn get_file_name_and_separator(arg: String) -> models::FileName {
    let open_parenthesis: Vec<_> = arg.match_indices("(").collect();
    let close_parenthesis: Vec<_> = arg.match_indices(")").collect();
    if open_parenthesis.len() != close_parenthesis.len() {
        panic!("invalid input");
    }
    let file_name_begin_index = open_parenthesis.get(0).expect("invalid");
    let file_name_end_index = close_parenthesis.get(0).expect("invalid");
    let file_name = &arg[file_name_begin_index.0 + 1..file_name_end_index.0];

    let separator_begin_index = open_parenthesis.get(1).expect("invalid");
    let separator_end_index = close_parenthesis.get(1).expect("invalid");
    let separator = &arg[separator_begin_index.0 + 1..separator_end_index.0];

    models::FileName {
        name: String::from(file_name),
        separator: String::from(separator),
    }
}

pub fn load_file(file_name: &str) -> String {
    let path = Path::new(file_name);
    let display = path.display();
    let error_message = format!("Failed to open file {}", display);
    let mut file = File::open(&path).expect(&error_message);
    let mut contents = String::new();
    let error_message = format!("Couldn't read content of file {}", display);
    file.read_to_string(&mut contents).expect(&error_message);
    contents
}

pub fn get_file_data<'a>(
    file_content: &'a str,
    separator: &'a str,
) -> Result<models::FileData, String> {
    let mut file_data = models::FileData::new();
    let mut header_names = Vec::new();
    let lines = file_content.lines().enumerate();
    for (index, line) in lines {
        if index == 0 {
            let first_line: Vec<&str> = line.split(separator).collect();
            for i in 0..first_line.len() {
                let current_header;
                if let Some(value) = first_line.get(i) {
                    current_header = String::from(value.clone());
                } else {
                    return Err("Error getting the headers".to_owned());
                }
                match file_data.data.get(&current_header) {
                    Some(_) => {
                        let error_message =
                            format!("Duplicated key found in the dataset: {}", current_header);
                        return Err(error_message);
                    }
                    None => {
                        header_names.push(current_header.to_string());
                        file_data.data.insert(current_header, Vec::new());
                    }
                }
                //data.insert(current_header, Vec::new());
            }
        } else {
            let values: Vec<&str> = line.split(separator).collect();
            if values.len() != file_data.data.keys().len() {
                let error_message = format!("Invalid data at line {}", index);
                return Err(error_message);
            }
            for i in 0..values.len() {
                if let Some(val) = file_data.data.get_mut(header_names.get(i).unwrap()) {
                    let current_data = values.get(i).unwrap();
                    val.push(current_data.to_string());
                }
            }
        }
    }
    Ok(file_data)
}

pub fn show_help() {
    println!("1-To load a file, enter the following command: load(your_file.ext).");
    println!("2-Exemple to request your data : select().cols(name, age). For more request examples, visit our documentation: http://dqs.io");
    println!("3-To exit the programm, enter the following command : q");
}
