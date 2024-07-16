use core::fmt;
use reqwest::Url;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_bencode;
use serde_json;
use std::{collections::BTreeMap, env};

#[derive(Deserialize, Serialize, Clone)]
struct Info {
    ///Size of the file to be downloaded in bytes, for single-torrent files
    length: usize,

    ///Suggested name to save the file/directory as
    name: String,

    ///Number of bytes in each piece
    #[serde(rename = "piece lenght")]
    piece_length: usize,

    ///Concatenated SHA-1 hashes of each piece
    piences: String,
}

///Represents a .torrent file, a metainfo file that contains a bencoded dictionary with keys and
///values
#[derive(Deserialize, Clone)]
struct Torrent {
    ///URL to a "tracker", which is a central server that keeps track of peers participating in the
    ///sharing of a torrent
    annouce: XUrl,
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

#[derive(Clone)]
struct XUrl(Url);

impl<'de> Deserialize<'de> for XUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // A Visitor that constructs an XUrl from a string URL
        struct XUrlVisitor;

        impl<'de> Visitor<'de> for XUrlVisitor {
            type Value = XUrl;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing an URL")
            }

            fn visit_str<E>(self, value: &str) -> Result<XUrl, E>
            where
                E: de::Error,
            {
                Url::parse(value)
                    .map(XUrl)
                    .map_err(|_| E::custom(format!("invalid URL: {}", value)))
            }
        }

        deserializer.deserialize_str(XUrlVisitor)
    }
}
