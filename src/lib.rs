pub mod lib {
    pub mod bencode;
    pub mod handshake;
    pub mod info;
    pub mod torrent;
}

pub use lib::bencode;
pub use lib::handshake;
pub use lib::info;
pub use lib::torrent;
