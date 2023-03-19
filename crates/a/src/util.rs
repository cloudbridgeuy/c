use bat::PrettyPrinter;
use bat::Syntax;
use chrono::prelude::*;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

const THEME: &str = "ansi";

fn lang_exists(lang: &String, langs: &Vec<Syntax>) -> bool {
    for l in langs {
        if &l.name.to_lowercase() == &lang.to_lowercase() {
            return true;
        }
        for e in &l.file_extensions {
            if e == &lang.to_lowercase() {
                return true;
            }
        }
    }
    false
}

pub fn pretty_print(str: &String, lang: &String) {
    let mut lang = lang.clone();
    let mut pp = PrettyPrinter::new();

    let langs: Vec<_> = pp.syntaxes().collect();
    if !lang_exists(&lang, &langs) {
        lang = String::from("txt");
    }

    pp.input_from_bytes(str.as_bytes())
        .language(&lang)
        .use_italics(true)
        .theme(THEME)
        .print()
        .unwrap();
}

pub fn copy_to_clipboard(str: &String) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(str.clone()).unwrap();
}

pub fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    // Open the file in write mode
    let mut file = File::create(file_path)?;

    // Write the content to the file
    file.write_all(content.as_bytes())?;

    Ok(())
}

pub fn read_file(file_path: &str) -> String {
    match File::open(file_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => contents,
                Err(_) => String::new(),
            }
        }
        Err(_) => String::new(),
    }
}

pub fn get_current_date() -> String {
    let local: DateTime<Local> = Local::now();
    local.format("%Y-%m-%d").to_string()
}

pub fn remove_code_lines(text: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        if !line.trim_start().starts_with("```") {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

pub fn create_dir_if_not_exist(dir_path: &str) -> std::io::Result<()> {
    if fs::metadata(dir_path).is_err() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())
}
