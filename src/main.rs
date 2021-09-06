use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    // TODO: faire le load_file

    let separator = ";";
    let data = load_file("./person_table.txt");
    let file_data = get_file_data(&data, separator).unwrap();
    //println!("{:#?}", file_data);

    //println!("{:#?}", arguments);

    let query = parse_arguments(arguments);
    let result = execute_query(query, &file_data);
    print_result(&result);
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

#[derive(Debug)]
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

fn parse_arguments(args: Vec<String>) -> String {
    if let Some(query) = args.get(1) {
        return query.to_string();
    } else {
        panic!("No query found. Enter a query in the command line to request data.");
    }
}

/**
 * Permet d'exécuter la requête
 */
fn execute_query<'a>(
    arg: String,
    file_data: &'a FileData,
) -> Result<HashMap<String, &'a Vec<String>>, String> {
    find_select(&arg).unwrap();
    execute_columns(&arg, file_data)
}

fn find_select(arg: &str) -> Result<String, String> {
    let regex = Regex::new(r"select\(\)").unwrap();
    match regex.find(&arg) {
        Some(_) => Ok("select()".to_string()),
        None => Err("Invalid query: select() not found.".to_owned()),
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
            for (key, values) in data.iter() {
                println!("{}", key);
                println!("----------");
                for i in 0..values.len() {
                    println!("{}-{}", i + 1, values[i]);
                }
                println!();
                println!("**************");
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
