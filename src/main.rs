use std::env;

use bittorrent_starter_rust::{bencode::decode_bencoded_value, torrent::Torrent};
//
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
        println!("{}", decoded_value.0);
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
