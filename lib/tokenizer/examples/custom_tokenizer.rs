//! You need to load the `encoder.json` and `vocab.bpe` in order to use this crate.
//!
//! A default `encoder.json` and `vocab.bpe` comes included in the library through
//! the `ENCODER_JSON` and `VOCAB_BPE` constants respectively. You may opt-out of
//! this variables by bringing your own files.
//!
//! The following example shows how you need to process this files in order to create
//! your `encode` and `decode` functions.

use std::iter::FromIterator;
use std::collections::HashMap;

use tokenizer::{ENCODER_JSON, VOCAB_BPE, bpe_ranks, bytes_to_unicode, decode, encode};

fn main() {
    let encoder: HashMap<String, u32> = serde_json::from_str(ENCODER_JSON).unwrap();
    let decoder: HashMap<u32, String> = HashMap::from_iter(encoder.clone().into_iter().map(|(k, v)| (v, k)));

    let lines: Vec<String> = VOCAB_BPE.lines().map(|line| line.to_owned()).collect();
    let bpe_ranks = bpe_ranks(&lines);

    let byte_encoder = bytes_to_unicode();
    let byte_decoder: HashMap<char, u32> = HashMap::from_iter(byte_encoder.clone().into_iter().map(|(k, v)| (v, k)));

    let text = r#"I'Many words map to one token, but some don't: indivisible.

Unicode characters like emojis may be split into many tokens containing the underlying bytes: 🤚🏾

Sequences of characters commonly found next to each other may be grouped together: 1234567890"#;
    let encoded = encode(&text, &bpe_ranks, &encoder);
    let decoded = decode(&encoded, &decoder, &byte_decoder);

    println!("Byte encoder: {:?}", byte_encoder);
    // println!("BPE Rank: {:?}", bpe_ranks);

    println!("Original text: {}", text);
    println!("Encoded text: {:#?}", encoded);
    println!("Decoded text: {}", decoded);

    println!("Text size: {}", text.len());
    println!("Words: {}", text.split(" ").count());
    println!("Rule of Thumb: {}", text.split(" ").count() * 4 / 3);
    println!("Tokens: {}", encoded.len());
}

