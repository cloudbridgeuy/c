//! This file includes code which was modified from https://github.com/openai/gpt-2
//! and https://github.com/latitudegames/GPT-3-Encoder/blob/master/Encoder.js
//! It was converted from JavaScript with the help of ChatGPT 4.0

use std::collections::{HashMap, HashSet};

use regex::Regex;

pub const ENCODER_JSON: &str = include_str!("encoder.json");
pub const VOCAB_BPE: &str = include_str!("vocab.bpe");

/// Default tokenizer that uses embedded encoder and vocab values to create the `encode` and
/// `decode` functions.
pub struct DefaultTokenizer {
    encoder: HashMap<String, u32>,
    decoder: HashMap<u32, String>,
    bpe_ranks: HashMap<Vec<String>, usize>,
    byte_decoder: HashMap<char, u32>,
}

impl DefaultTokenizer {
    /// Creates a new DefaultTokenizer.
    pub fn new() -> Self {
        let byte_encoder = bytes_to_unicode();
        let lines: Vec<String> = VOCAB_BPE.lines().map(|line| line.to_owned()).collect();
        let encoder: HashMap<String, u32> = serde_json::from_str(ENCODER_JSON).unwrap();

        Self {
            encoder: encoder.clone(),
            decoder: HashMap::from_iter(encoder.clone().into_iter().map(|(k, v)| (v, k))),
            bpe_ranks: bpe_ranks(&lines),
            byte_decoder: HashMap::from_iter(byte_encoder.clone().into_iter().map(|(k, v)| (v, k))),
        }
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        encode(&text, &self.bpe_ranks, &self.encoder)
    }

    pub fn decode(&self, encoded: &[u32]) -> String {
        decode(&encoded, &self.decoder, &self.byte_decoder)
    }
}

/// Constructs the `bpe_ranks` hashmap from a `vocab.bpe` file provides as a list of lines.
pub fn bpe_ranks(lines: &[String]) -> HashMap<Vec<String>, usize> {
    let bpe_merges: Vec<Vec<String>> = lines
        .iter()
        .map(|x| x.split_whitespace().map(|s| s.to_owned()).collect())
        .collect();

    dict_zip(&bpe_merges, &(0..bpe_merges.len()).collect::<Vec<usize>>())
}

/// Constructs a bytes to unicode HashMap.
pub fn bytes_to_unicode() -> HashMap<u32, char> {
    let mut bs = range(ord('!'), ord('~') + 1)
        .iter()
        .chain(range(ord('¡'), ord('¬') + 1).iter())
        .chain(range(ord('®'), ord('ÿ') + 1).iter())
        .cloned()
        .collect::<Vec<u32>>();

    let mut cs = bs.clone();
    let mut n = 0;
    for b in 0..(2_u32.pow(8)) {
        if !bs.contains(&b) {
           bs.push(b);
           cs.push(2_u32.pow(8) + n);
           n += 1;
        }
    }

    let cs: Vec<char> = cs.into_iter().map(chr).collect();

    dict_zip(&bs, &cs)
}

/// Encodes a string using a custom bpe_ranks and encoder HashMaps.
pub fn encode(text: &str, bpe_ranks: &HashMap<Vec<String>, usize>, encoder: &HashMap<String, u32>) -> Vec<u32> {
    // I had to update this regex to makr it work in Rust, given that Rust doesn't support
    // look-around assertions.
    //
    // - `'s|'t|'re|'ve|'m|'ll|'d: This part of the regex matches common contractions in English, such as 's, 't, 're, 've, 'm, 'll, and 'd.
    // - `?\p{L}+: This part matches Unicode letters (L) with an optional space (?) before them. The plus sign (+) indicates one or more occurrences of the preceding element.
    // - `?\p{N}+: This part matches Unicode numbers (N) with an optional space (?) before them. The plus sign (+) indicates one or more occurrences of the preceding element.
    // - `?[^\s\p{L}\p{N}]+: This part matches any character that is not a whitespace (\s), letter (\p{L}), or number (\p{N}) with an optional space (?) before them. The plus sign (+) indicates one or more occurrences of the preceding element.
    // - `\s+: This part matches one or more whitespace characters (\s+).
    let pat = Regex::new(r"'s|'t|'re|'ve|'m|'ll|'d| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+$|\s+").unwrap();
    let mut bpe_tokens = Vec::new();

    for token in pat.find_iter(text) {
        let token = token.as_str();
        let token = encode_str(token);
        let token = token.into_iter().map(|x| chr(x.parse::<u32>().unwrap()).to_string()).collect::<Vec<_>>().join("");

        let new_tokens: Vec<u32> = bpe(&token, bpe_ranks).split_whitespace().map(|x| encoder[x]).collect();
        bpe_tokens.extend(new_tokens);
    }

    bpe_tokens
}


/// Decodes an encoded string using a custom decoder and byte decoder created from the encoder that
/// encoded the original string.
pub fn decode(tokens: &[u32], decoder: &HashMap<u32, String>, byte_decoder: &HashMap<char, u32>) -> String {
    let text: String = tokens.iter().map(|x| decoder[x].as_str()).collect();
    let text: String = text.chars().map(|x| chr(*byte_decoder.get(&x).unwrap())).collect();

    text
}

fn range(x: u32, y: u32) -> Vec<u32> {
    (x..y).collect()
}

fn ord(ch: char) -> u32 {
    ch as u32
}

fn chr(code: u32) -> char {
    char::from_u32(code).unwrap()
}

fn encode_str(s: &str) -> Vec<String> {
    s.as_bytes().iter().map(|b| b.to_string()).collect()
}

fn dict_zip<T: Eq + std::hash::Hash + Clone, U: Clone>(x: &[T], y: &[U]) -> HashMap<T, U> {
    let mut map = HashMap::new();
    for (i, key) in x.iter().enumerate() {
       map.insert(key.clone(), y[i].clone());
    }
    map
}

fn get_pairs(word: &[String]) -> HashSet<Vec<String>> {
    let mut pairs = HashSet::new();
    let mut prev_char = &word[0];
    for ch in word.iter().skip(1) {
        pairs.insert(vec![prev_char.clone(), ch.clone()]);
        prev_char = ch;
    }
    pairs
}

fn bpe(token: &str, bpe_ranks: &HashMap<Vec<String>, usize>) -> String {
    let byte_encoder = bytes_to_unicode();

    let mut word = token.chars().map(|c| byte_encoder[&(c as u32)].to_string()).collect::<Vec<_>>();
    let mut pairs = get_pairs(&word);

    while !pairs.is_empty() {
        let min_pair_rank = pairs
            .iter()
            .map(|pair| bpe_ranks.get(pair).copied().unwrap_or(usize::MAX))
            .min()
            .unwrap();
        let bigram = pairs
            .iter()
            .find(|pair| bpe_ranks.get(*pair).copied().unwrap_or(usize::MAX) == min_pair_rank)
            .cloned()
            .unwrap();

        if !bpe_ranks.contains_key(&bigram) {
            break
        }

        let first = &bigram[0];
        let second = &bigram[1];
        let mut new_word = Vec::new();
        let mut i = 0;

        while i < word.len() {
            let j = word[i..].iter().position(|x| x == first);
            if let Some(j) = j {
                new_word.extend_from_slice(&word[i..i + j]);
                i += j;

                if i < word.len() - 1 && &word[i + 1] == second {
                    new_word.push(format!("{}{}", first, second));
                    i += 2;
                } else {
                    new_word.push(word[i].clone());
                    i += 1;
                }
            } else {
                new_word.extend_from_slice(&word[i..]);
                break;
            }
        }

        word = new_word;
        pairs = get_pairs(&word);
    }

    word.join(" ")
}
