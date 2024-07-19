use std::fs;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::info::Info;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Torrent {
    ///The URL of the tracker, which is a central server that keeps track of peers participating in
    ///the sharing of the torrent.
    pub announce: String, //TODO: change to URL

    pub info: Info,
}

impl Torrent {
    ///Generate Torrent from a file path
    ///The file must have bencoded data
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let torrent_bytes = fs::read(path)?;
        let torrent = serde_bencode::de::from_bytes::<Torrent>(&torrent_bytes)?;

        Ok(torrent)
    }

    ///Get Torrent's info
    pub fn get_info(&self) -> Info {
        self.info.clone()
    }

    ///Get a SHA1 hex of the torrent's info dictionary
    pub fn get_sha1(&self) -> Result<[u8; 20], Box<dyn std::error::Error>> {
        let encoded_bytes = serde_bencode::to_bytes(&self.info)?;
        let encoded_bytes_ref: &[u8] = encoded_bytes.as_ref();

        let mut hasher = Sha1::new();
        hasher.update(encoded_bytes_ref);

        let result = hasher.finalize();

        Ok(result.try_into().expect("Generic Array"))
    }

    pub fn print_pieces_info(&self) {
        println!("Piece Length: {}", self.info.piece_length);
        println!("Piece Hashes:");

        for piece in &self.info.pieces.0 {
            println!("{}", hex::encode(piece));
        }
    }

    pub async fn discover_peers(&self, info_hash: &[u8; 20]) {
        let client = reqwest::Client::new();

        let query_params = [
            ("peer_id", "00112233445566778899".to_string()),
            ("port", "6881".to_string()),
            ("uploaded", 0.to_string()),
            ("downloaded", 0.to_string()),
            ("left", self.info.length.to_string()),
            ("compact", 1.to_string()),
        ];

        println!("{:?}", query_params);

        let url = format!(
            "{}?{}&info_hash={}",
            &self.announce,
            serde_urlencoded::to_string(query_params).unwrap(),
            urlencode(info_hash)
        );

        println!("{}", url);

        let response = client.get(url).send().await.unwrap();

        println!("{:?}", response);
    }
}

fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());
    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }
    encoded
}
