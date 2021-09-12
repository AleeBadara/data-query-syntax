use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
mod messages;
mod models;
mod utils;
mod validators;

// TODO:
// créer un dossier validators/mod.rs. il va contenir les méthodes verify_load_input, verify_select_input et d'autres dans le futur
// créer dossier queries/mod.rs
// créer un dossier messages/mod.rs qui va contenir les println
// créer un dossier printer/mod.rs qui va contenir la fonctionnalité qui permet d'afficher les résultats

fn main() {
    messages::show_welcome_title();
    let mut file_data: models::FileData = Default::default();
    loop {
        messages::show_welcome_message();
        let user_query = utils::get_user_input();
        match user_query.trim() {
            "q" => {
                messages::show_programm_end_message();
                break;
            }
            "h" => utils::show_help(),
            _ => match find_query(&user_query) {
                models::Command::LOAD => {
                    validators::verify_load_input(&user_query);
                    file_data = execute_load_query(user_query).unwrap();
                    messages::show_load_success_message();
                }
                models::Command::SELECT => {
                    let result = execute_query(user_query, &file_data);
                    print_result(&result);
                }
                models::Command::UMBIGUOUS => {
                    messages::show_umbiguous_message();
                }
                models::Command::UNKNOWN => {
                    messages::show_unknown_command_message();
                }
            },
        }
    }
}

fn find_query(arg: &str) -> models::Command {
    if arg.contains("load") && arg.contains("select") == false {
        return models::Command::LOAD;
    }
    if arg.contains("select") && arg.contains("load") == false {
        return models::Command::SELECT;
    }
    if arg.contains("select") && arg.contains("load") {
        return models::Command::UMBIGUOUS;
    }
    models::Command::UNKNOWN
}

fn execute_load_query(arg: String) -> Result<models::FileData, String> {
    let file_meta_data = utils::get_file_name_and_separator(arg);
    let data = utils::load_file(&file_meta_data.name);
    utils::get_file_data(&data, &file_meta_data.separator)
}

/**
 * Permet d'exécuter la requête
 */
fn execute_query<'a>(
    arg: String,
    file_data: &'a models::FileData,
) -> Result<HashMap<String, &'a Vec<String>>, String> {
    validators::verify_select_input(&arg);
    execute_columns(&arg, file_data)
}

fn execute_columns<'a>(
    arg: &str,
    file_data: &'a models::FileData,
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
