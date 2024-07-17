use serde::{Deserialize, Serialize};
use serde_bencode;
use serde_json;
use std::env;

#[derive(Clone, Deserialize, Serialize)]
struct Info {
    ///Suggested name (or directory if multifile) to save as.
    name: Vec<u8>,

    ///Number of bytes in each piece the file is split into. Files are split into fixed-size lenght
    ///pieces which are all the same except for the last one, which may be truncated. It is almost
    ///always a power of two, most commonly 2 18 = 256K.
    #[serde(rename = "piece length")]
    piece_length: u64,

    ///String with length multiple of 20. It is subdivided into strings with length of 20, each of
    ///which is the SHA1 hash of the piece at the corresponding index.
    pieces: Vec<u8>,

    ///Length of a single file, in number of bytes
    length: u64,

    ///A list of UTF-8 encoded strings corresponding to subdirectory names, the last of which is
    ///the actual file name (a zero length list is an error case)
    path: Vec<Vec<u8>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Torrent {
    ///The URL of the tracker, which is a central server that keeps track of peers participating in
    ///the sharing of the torrent.
    announce: Vec<u8>, //TODO: change to URL

    info: Info,
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
        Some('d') => {
            let mut dict = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;

            while !rest.is_empty() && !rest.starts_with('e') {
                let (k, remainder) = decode_bencoded_value(rest);

                let k = match k {
                    serde_json::Value::String(k) => k,
                    k => panic!("key must be string, not {k:?}"),
                };

                let (v, remainder) = decode_bencoded_value(remainder);
                dict.insert(k, v);
                rest = remainder;
            }

            return (dict.into(), &rest[1..]);
        }
        _ => {}
    }

    panic!("Unhandled encoded value: {}", encoded_value)
}
