use std::{
    env,
    io::Write,
    net::{SocketAddrV4, TcpStream},
    str::FromStr,
};

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
                }
                Err(e) => {
                    println!("Failed to parse: {e}");
                }
            }
        }
        "peers" => {
            let file_path = &args[2];

            match Torrent::from_file(file_path) {
                Ok(torrent) => {
                    let sha_hex = torrent
                        .get_sha1()
                        .expect("encoding after successfil decode is ok");

                    match torrent.discover_peers(&sha_hex).await {
                        Ok(trackers) => {
                            trackers.peers.0.iter().for_each(|addr| {
                                println!("{addr}");
                            });
                        }
                        Err(e) => {
                            println!("{e}");
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to parse: {e}");
                }
            }
        }
        "handshake" => {
            let addr_str = &args[3];

            let addr = match SocketAddrV4::from_str(addr_str) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("Failed to parse addr: {e}");
                    return;
                }
            };

            if let Ok(mut stream) = TcpStream::connect(addr) {
                println!("Connected to peer");
                let b: &[u8] = &19_i32.to_le_bytes();
                let s = stream.write(b);
                println!("{s:?}");
            }
        }
        _ => {
            eprintln!("unknown command: {}", args[1])
        }
    }
}
