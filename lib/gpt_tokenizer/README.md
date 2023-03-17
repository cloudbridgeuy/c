# GPT-Tokenizer

An implementation of the GPT-3 tokenizer created by converting the [`GPT-3-Encoder`](https://www.npmjs.com/package/gpt-3-encoder)
JavaScript package to Rust (with the help of ChatGPT-4). You can use it to estimate the number of
tokens that your prompt would approximately consume. You can also create your own custom `encoding` and
`decoding` functions by providing your own `encoder.json` and `vocab.bpe` files.

use tokenizer::DefaultTokenizer;

fn main() {
    let tokenizer = DefaultTokenizer::new();

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

See the [./examples](./examples) directory to see more examples of how to use it.
