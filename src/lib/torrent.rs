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

    ///Print Torrent's tracker URL and length
    pub fn print_info(&self) {
        println!("Tracker URL: {}", self.announce);
        println!("Length: {}", self.info.length);
    }

    ///Print a SHA1 hex of the torrent's info dictionary
    pub fn print_sha1_hex(&self) -> Result<(), Box<dyn std::error::Error>> {
        let encoded_bytes = serde_bencode::to_bytes(&self.info)?;
        let encoded_bytes_ref: &[u8] = encoded_bytes.as_ref();

        let mut hasher = Sha1::new();
        hasher.update(encoded_bytes_ref);

        let result = hasher.finalize();
        println!("Info Hash: {}", hex::encode(result));

        Ok(())
    }

    pub fn print_pieces_info(&self) {
        println!("Piece Length: {}", self.info.piece_length);
        println!("Piece Hashes:");

        let piece_vec: Vec<&[u8]> = self.info.pieces.chunks(20).collect();

        for piece in piece_vec {
            println!("{}", hex::encode(piece));
        }
    }
}
