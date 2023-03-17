//! The library comes with a DefaultTokenizer, which is a struct that loads the internal
//! `encoder.json` and `vocab.bpe`. It simplifies the creation of the `encode` and `decode`
//! functions. This is specially useful when you just want to estimate the number of tokens
//! your prompt will consume.
//!
//! > As a rule of thumb, OpenAI suggest that 100 tokens equal 75 words.
use gpt_tokenizer::Default;

fn main() {
    let tokenizer = Default::new();

    let text = r#"I'Many words map to one token, but some don't: indivisible.

Unicode characters like emojis may be split into many tokens containing the underlying bytes: 🤚🏾

Sequences of characters commonly found next to each other may be grouped together: 1234567890"#;

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

