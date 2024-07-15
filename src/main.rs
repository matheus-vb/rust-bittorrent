use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    // If encoded_value starts with a digit, it's a number
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((num, rest)) =
                encoded_value
                    .split_at(1)
                    .1
                    .split_once('e')
                    .and_then(|(num, rest)| {
                        let n = num.parse::<i64>().ok()?;
                        Some((n, rest))
                    })
            {
                return (num.into(), rest);
            }
        }
        Some('0'..='9') => {
            if let Some((len, text)) = encoded_value.split_once(':') {
                if let Ok(len) = len.parse::<usize>() {
                    return (text[..len].to_string().into(), &text[len..]);
                }
            }
        }
        Some('l') => {
            let mut elements = Vec::new();
            let mut rest = encoded_value.split_at(1).1;

            while !rest.is_empty() && !rest.starts_with('e') {
                let (e, remainder) = decode_bencoded_value(rest);
                elements.push(e);
                rest = remainder;
            }

            return (elements.into(), &rest[1..]);
        }
        _ => {}
    }

    panic!("Unhandled encoded value: {}", encoded_value)
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0.to_string());
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}
