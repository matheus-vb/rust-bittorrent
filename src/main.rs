use std::env;

use bittorrent_starter_rust::{bencode::decode_bencoded_value, torrent::Torrent};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value);
            println!("{}", decoded_value.0);
        }
        "info" => {
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
        }
        _ => {
            eprintln!("unknown command: {}", args[1])
        }
    }
}
