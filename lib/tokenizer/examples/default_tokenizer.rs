//! The library comes with a DefaultTokenizer, which is a struct that loads the internal
//! `encoder.json` and `vocab.bpe`. It simplifies the creation of the `encode` and `decode`
//! functions. This is specially useful when you just want to estimate the number of tokens
//! your prompt will consume.
//!
//! > As a rule of thumb, OpenAI suggest that 100 tokens equal 75 words.
use tokenizer::DefaultTokenizer;

fn main() {
    let tokenizer = DefaultTokenizer::new();

    let text = "I'm a string with some contractions like I'm, you're, and we'll, as well as some numbers like 123 and some punctuation like !?";
    let encoded = &tokenizer.encode(text);
    let decoded = &tokenizer.decode(encoded);

    println!("Original text: {}", text);
    println!("Encoded text: {:#?}", encoded);
    println!("Decoded text: {}", decoded);

    println!("Text size: {}", text.len());
    println!("Words: {}", text.split(" ").count());
    println!("Rule of Thumb: {}", text.split(" ").count() * 4 / 3);
    println!("Tokens: {}", encoded.len());
}

