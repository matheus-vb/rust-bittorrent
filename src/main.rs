use std::{env, mem, net::SocketAddrV4, str::FromStr};

use bittorrent_starter_rust::{
    bencode::decode_bencoded_value, handshake::Handshake, torrent::Torrent,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
            let file_path = &args[2];
            let peer_addr = &args[3];

            let torrent = match Torrent::from_file(file_path) {
                Ok(torrent) => torrent,
                Err(e) => {
                    println!("{e}");
                    return;
                }
            };

            let info_hash = torrent.get_sha1().expect("should work");

            let addr = match SocketAddrV4::from_str(peer_addr) {
                Ok(addr) => addr,
                Err(e) => {
                    println!("Failed to parse addr: {e}");
                    return;
                }
            };

            let mut peer = match tokio::net::TcpStream::connect(addr).await {
                Ok(peer) => peer,
                Err(e) => {
                    println!("{e}");
                    return;
                }
            };

            let mut handshake = Handshake::new(info_hash, *b"00112233445566778899");
            {
                let handshake_bytes =
                    &mut handshake as *mut Handshake as *mut [u8; mem::size_of::<Handshake>()];

                //Safety: Handshake is a repr(C)
                let handshake_bytes: &mut [u8; mem::size_of::<Handshake>()] =
                    unsafe { &mut *handshake_bytes };

                if let Err(e) = peer.write_all(handshake_bytes).await {
                    println!("{e}");
                    return;
                }

                if let Err(e) = peer.read_exact(handshake_bytes).await {
                    println!("{e}");
                    return;
                }

                println!("Peer ID: {}", hex::encode(&handshake.peer_id));
            }
        }
        _ => {
            eprintln!("unknown command: {}", args[1])
        }
    }
}
