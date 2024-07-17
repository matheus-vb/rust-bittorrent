use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use serde_bencode;
use serde_json;
use sha1::{Digest, Sha1};
use std::{env, error::Error, fs};

#[derive(Clone, Deserialize, Serialize, Debug)]
struct Info {
    ///Suggested name (or directory if multifile) to save as.
    name: String,

    ///Number of bytes in each piece the file is split into. Files are split into fixed-size lenght
    ///pieces which are all the same except for the last one, which may be truncated. It is almost
    ///always a power of two, most commonly 2 18 = 256K.
    #[serde(rename = "piece length")]
    piece_length: usize,

    ///String with length multiple of 20. It is subdivided into strings with length of 20, each of
    ///which is the SHA1 hash of the piece at the corresponding index.
    #[serde(
        deserialize_with = "deserialize_pieces",
        serialize_with = "serialize_pieces"
    )]
    pieces: Vec<u8>,

    ///Length of a single file, in number of bytes
    length: usize,

    ///A list of UTF-8 encoded strings corresponding to subdirectory names, the last of which is
    ///the actual file name (a zero length list is an error case)
    path: Option<Vec<Vec<String>>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct Torrent {
    ///The URL of the tracker, which is a central server that keeps track of peers participating in
    ///the sharing of the torrent.
    announce: String, //TODO: change to URL

    info: Info,
}

impl Torrent {
    ///Generate Torrent from a file path
    ///The file must have bencoded data
    fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let torrent_bytes = fs::read(path)?;
        let torrent = serde_bencode::de::from_bytes::<Torrent>(&torrent_bytes)?;

        Ok(torrent)
    }

    ///Print Torrent's tracker URL and length
    fn print_info(&self) {
        println!("Tracker URL: {}", self.announce);
        println!("Length: {}", self.info.length);
    }

    ///Print a SHA1 hex of the torrent's info dictionary
    fn print_sha1_hex(&self) -> Result<(), Box<dyn Error>> {
        let encoded_bytes = serde_bencode::to_bytes(&self.info)?;
        let encoded_bytes_ref: &[u8] = encoded_bytes.as_ref();

        let mut hasher = Sha1::new();
        hasher.update(encoded_bytes_ref);

        let result = hasher.finalize();
        println!("Info Hash: {}", hex::encode(result));

        Ok(())
    }

    fn print_pieces_info(&self) {
        println!("Piece Lenght: {}", self.info.piece_length);
        println!("Piece hashes:");

        let piece_vec: Vec<&[u8]> = self.info.pieces.chunks(20).collect();

        for piece in piece_vec {
            println!("{}", hex::encode(piece));
        }
    }
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
    } else if command == "info" {
        let file_path = &args[2];

        match Torrent::from_file(file_path) {
            Ok(torrent) => {
                torrent.print_info();

                torrent
                    .print_sha1_hex()
                    .expect("encoding after successful decoding should be ok");

                torrent.print_pieces_info();
            }
            Err(e) => {
                println!("Failed to parse: {e}");
            }
        }
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

fn deserialize_pieces<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecU8Visitor;

    impl<'de> Visitor<'de> for VecU8Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a byte string")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_vec())
        }
    }

    deserializer.deserialize_bytes(VecU8Visitor)
}

fn serialize_pieces<S>(pieces: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(pieces.as_slice())
}
