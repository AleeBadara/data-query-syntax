use colored::*; // lib pour afficher du texte en couleur dans le terminal (https://crates.io/crates/colored)
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    println!("{}", "Welcome to DQS (Data Query Syntax).".green());
    let mut file_data: FileData = Default::default();
    loop {
        show_welcome_message();
        let user_query = get_user_input();
        match &user_query.trim()[..] {
            "q" => {
                println!("{}", "Program ended.".green());
                break;
            }
            "h" => show_help(),
            _ => {
                if user_query.contains("load") {
                    verify_load_input(&user_query);
                    file_data = execute_load_query(&user_query).unwrap();
                    println!("{}", "Data loaded successfully.".green());
                }
                if user_query.contains("select") {
                    let result = execute_query(user_query, &file_data);
                    print_result(&result);
                }
            }
        }
    }
    //let arguments: Vec<String> = env::args().collect();
}

fn show_welcome_message() {
    println!();
    println!(
        "{}",
        ">Load your data file and enter queries to request it.".blue()
    );
    println!("{}", ">Enter h for help or q to quit.".blue());
}

fn get_user_input() -> String {
    let mut user_input = String::new();
    io::stdin()
        .lock()
        .read_line(&mut user_input)
        .expect("Unable to read your input.");
    user_input
}

fn show_help() {
    println!("1-To load a file, enter the following command: load(your_file.ext).");
    println!("2-Exemple to request your data : select().cols(name, age). For more request examples, visit our documentation: http://dqs.io");
    println!("3-To exit the programm, enter the following command : q");
}

fn execute_load_query(arg: &str) -> Result<FileData, String> {
    let file_meta_data = get_file_name_and_separator(arg);
    let data = load_file(&file_meta_data.name);
    get_file_data(&data, &file_meta_data.separator)
}

fn get_file_name_and_separator(arg: &str) -> FileName {
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

    FileName {
        name: String::from(file_name),
        separator: String::from(separator),
    }
}

fn load_file(file_name: &str) -> String {
    let path = Path::new(file_name);
    let display = path.display();
    let error_message = format!("Failed to open file {}", display);
    let mut file = File::open(&path).expect(&error_message);
    let mut contents = String::new();
    let error_message = format!("Couldn't read content of file {}", display);
    file.read_to_string(&mut contents).expect(&error_message);
    contents
}

#[derive(Debug, Default)]
struct FileData {
    data: HashMap<String, Vec<String>>,
}

impl FileData {
    fn new() -> Self {
        FileData {
            data: HashMap::new(),
        }
    }
}

struct FileName {
    name: String,
    separator: String,
}

fn get_file_data<'a>(file_content: &'a str, separator: &'a str) -> Result<FileData, String> {
    let mut file_data = FileData::new();
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

/**
 * Permet d'exécuter la requête
 */
fn execute_query<'a>(
    arg: String,
    file_data: &'a FileData,
) -> Result<HashMap<String, &'a Vec<String>>, String> {
    verify_select_input(&arg);
    execute_columns(&arg, file_data)
}

fn verify_load_input(arg: &str) {
    let regex_load = Regex::new(r"load\(.+\..+\)\.separator\(.{1}\)").unwrap();
    match regex_load.find(&arg) {
        Some(_) => (),
        None => panic!("Invalid query. See how to use the load method in the documentation."),
    }
}

fn verify_select_input(arg: &str) {
    let regex = Regex::new(r"select\(\)").unwrap();
    match regex.find(&arg) {
        Some(_) => (),
        None => panic!("Invalid query. See how to use the select method in the documentation."),
    }
}

fn execute_columns<'a>(
    arg: &str,
    file_data: &'a FileData,
) -> Result<HashMap<String, &'a Vec<String>>, String> {
    let col_regex = Regex::new(r"\.cols\(\**\)").unwrap();
    let mut temp_result: HashMap<String, &Vec<String>> = HashMap::new();
    match col_regex.find(&arg) {
        Some(_) => {
            for (key, val) in file_data.data.iter() {
                temp_result.insert(key.to_string(), &val);
            }
            return Ok(temp_result);
        }
        None => {
            let col_regex = Regex::new(r"\.cols\((.*)+?\)").unwrap();
            match col_regex.find(&arg) {
                Some(result) => {
                    let columns_substring = &arg[result.start()..result.end()];
                    let columns = &columns_substring[columns_substring.find('(').unwrap() + 1
                        ..columns_substring.find(')').unwrap()];
                    let columns_array: Vec<&str> = columns.split(',').collect();
                    for i in 0..columns_array.len() {
                        if let Some(val) = file_data.data.get(columns_array[i]) {
                            temp_result.insert(columns_array[i].to_string(), val);
                        }
                    }
                    Ok(temp_result)
                }
                None => {
                    return Err("Invalid query: cols not found or invalid.".to_owned());
                }
            }
        }
    }
}

fn print_result(result: &Result<HashMap<String, &Vec<String>>, String>) {
    match result {
        Ok(data) => {
            println!();
            for (key, values) in data.iter() {
                println!("-- {} --", key);
                for i in 0..values.len() {
                    println!("{}-{}", i + 1, values[i]);
                }
                println!();
            }
        }
        Err(_) => {}
    }
}

// NB: on n'a pas de .from("table") dans les requêtes en dessous car on load le fichier contenant les data. donc pas besoin de table
// select().cols("*")
// select().cols("*").limit(5)
// select().cols("*").limit(5).offset(10)
// select().cols("nom","prenom")
// select().cols("*").equal("nom","john")
// select().cols(""nom","prenom"").equal("nom","john").equal("prenom","doe")
// select().cols("*").any("prenom",["jane,meri"])
// select().cols("*").in("age",["18,19,30"])
// select().cols("*").not("prenom", "jane")
// select().cols("*").gt("age", "18")
// select().cols("*").gte("age", "18")
// select().cols("*").lt("age", "18")
// select().cols("*").lte("age", "18")
// select().cols("*").between("age", "18", "30")
// select().cols("*").any_like("prenom",["j%,%s"])
// select().cols("*").like("prenom","toto")

// Ordre d'exécution que je vais appliquer:
// 1- cols => pour récupérer les colonnes
// 2- conditions

// select().from("person_table").equal("nom","john").any("prenom"["jane,meri"]).like("prenom","toto")
// select().from("person_table").equal("nom","john").any("prenom"["jane,meri"]).like("prenom","toto")

/*macro_rules! custom_sql {
    ($data:ident, select().cols(*)) => {
        $data
    };
    ($data:ident, select().cols($($cols:literal),*))=>{{
        let mut result:HashMap<String, Vec<String>> = HashMap::new();
        $(
            if let Some(column_values) = $data.get($cols){
                if let Some(_)= result.get_mut($cols){
                }else{
                    let mut filtered_data:Vec<String> = Vec::new();
                    for i in 0..column_values.len(){
                        filtered_data.push(column_values.get(i).unwrap().clone().to_string());
                    }
                    result.insert($cols.to_string(), filtered_data);
                }
            }else{
                let error_message = format!("Key :{} not found in the dataset", $cols);
                panic!("{}",error_message);
            }
        )*
        result
    }};
    (select().cols($($cols:literal),* $(*)?)$(.equal($key:literal,$val:literal))*)=>{
        println!("{}", $table);
        $(
            println!("cols :{}", $cols);
        )*
        $(
            println!("Key/Value: {}/{}", $key, $val);
        )*
    };
    (select().cols($($cols:literal),* $(*)?)$(.any($key:literal,[$($vals:literal),*]))*)=>{
        println!("{}", $table);
        $(
            println!("cols :{}", $cols);
        )*
        $(
            println!("Key: {}", $key);
        )*
        $(
            $(
                println!("Values : {}", $vals);
            )*
        )*
    };
    (select $($cols:literal),* from $table:literal where $($c:literal $(in ($($vals:literal),*))? $(= $val:literal)?  $(and)? $(or)?)*) => {
        $(
            println!("{}",$cols);
        )*
        $(
            $(
                println!("{}",$val);
            )*
        )*
        $(
            $(
                $(
                    println!("{}",$vals);
                )*
            )*
        )*
    };
    (select $($cols:literal),* from $table:literal where $($c:literal = $val:literal $(and)? $(or)?)* $($c2:literal in [$($val2:literal),*])* ) => {
        $(
            println!("{}",$cols);
        )*
        $(
            println!("{}",$val);
        )*
    };
}*/

/*println!(
    "{:#?}",
    custom_sql!(file_data,select().from("person_table").cols(*))
);

println!("--------------------------------------");

println!(
    "{:#?}",
    custom_sql!(file_data, select().cols("nom", "prenom"))
);*/

/*println!("--------------------------------------");

custom_sql!(select()
    .from("person_table")
    .cols("nom", "prenom")
    .equal("nom", "toto"));
custom_sql!(select().from("person_table").cols("*").equal("nom", "tutu"));

println!("--------------------------------------");

custom_sql!(select()
    .from("person_table")
    .cols("nom", "prenom")
    .equal("nom", "toto")
    .equal("prenom", "titi"));

println!("--------------------------------------");

custom_sql!(select()
    .from("person_table")
    .cols("nom", "prenom")
    .any("nom", ["toto", "tata"])
    .any("prenom", ["tete", "titi"]));*/
