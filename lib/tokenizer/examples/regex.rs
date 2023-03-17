use regex::Regex;

fn main() {
    // let re1 = Regex::new(r#"s[|']t|[|']re|[|']ve|[|']m|[|']ll|[|']d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+"#).unwrap();
    // let re2 = Regex::new(r#"s[|']t|[|']re|[|']ve|[|']m|[|']ll|[|']d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+$|\s+"#).unwrap();
    // let re1 = Regex::new(r"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+").unwrap();
    let re1 = Regex::new(r"'s|'t|'re|'ve|'m|'ll|'d| ?\pL+| ?\pN+| ?[^\s\pL\pN]+|\s+$|\s+").unwrap();

    let text = "I'm a string with some contractions like I'm, you're, and we'll, as well as some numbers like 123 and some punctuation like !?";

    println!("Parsed text with re1");
    for cap in re1.captures_iter(text) {
        println!("{:?}", cap);
    }
}

