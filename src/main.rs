use std::env;

use bittorrent_starter_rust::{bencode::decode_bencoded_value, torrent::Torrent};

#[tokio::main]
async fn main() {
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
                    let info = torrent.get_info();

                    println!("Tracker URL: {}", torrent.announce);
                    println!("Length: {}", info.length);

                    let sha_hex = torrent
                        .get_sha1()
                        .expect("encoding after successful decoding should be ok");

                    println!("Info Hash: {}", hex::encode(sha_hex));

                    torrent.print_pieces_info();

                    torrent.discover_peers(&sha_hex).await;
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
