use std::fmt::Display;

pub fn format_vector<T: Display>(vec: &Vec<T>) -> String {
    let formatted = vec
        .iter()
        .map(|item| item.to_string()) // convert each item to string
        .collect::<Vec<String>>()     // collect into a Vec<String>
        .join(", ");                  // join with commas
    format!("[{}]", formatted)       // wrap in brackets
}