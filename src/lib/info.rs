use std::net::{Ipv4Addr, SocketAddrV4};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Info {
    ///Suggested name (or directory if multifile) to save as.
    pub name: String,

    ///Number of bytes in each piece the file is split into. Files are split into fixed-size lenght
    ///pieces which are all the same except for the last one, which may be truncated. It is almost
    ///always a power of two, most commonly 2^18 = 256K.
    #[serde(rename = "piece length")]
    pub piece_length: usize,

    ///String with length multiple of 20. It is subdivided into strings with length of 20, each of
    ///which is the SHA1 hash of the piece at the corresponding index.
    pub pieces: Pieces,

    ///Length of a single file, in number of bytes
    pub length: usize,

    ///A list of UTF-8 encoded strings corresponding to subdirectory names, the last of which is
    ///the actual file name (a zero length list is an error case)
    pub path: Option<Vec<Vec<String>>>,
}

#[derive(Clone, Debug)]
pub struct Pieces(pub Vec<[u8; 20]>);
struct PiecesVisitor;

impl<'de> Visitor<'de> for PiecesVisitor {
    type Value = Pieces;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte string with a lenght multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() % 20 != 0 {
            return Err(E::custom(format!(
                "length {} is not multiple of 20",
                v.len()
            )));
        }

        let pieces: Vec<[u8; 20]> = v
            .chunks(20)
            .map(|slice| slice.try_into().expect("slice length is 20"))
            .collect();

        Ok(Pieces(pieces))
    }
}

impl<'de> Deserialize<'de> for Pieces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PiecesVisitor)
    }
}

impl Serialize for Pieces {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let slice = self.0.concat();
        serializer.serialize_bytes(&slice)
    }
}

#[derive(Debug, Clone)]
pub struct Peers(pub Vec<SocketAddrV4>);
struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter
            .write_str("6 bytes, the first 4 being the IP address, and the last 2 the port number")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::custom(format!(
                "length {} is not multiple of 6",
                v.len()
            )));
        }

        let peers: Vec<SocketAddrV4> = v
            .chunks_exact(6)
            .map(|slice| {
                SocketAddrV4::new(
                    Ipv4Addr::new(slice[0], slice[1], slice[2], slice[3]),
                    u16::from_be_bytes([slice[4], slice[5]]),
                )
            })
            .collect();

        Ok(Peers(peers))
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

impl Serialize for Peers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut slice = Vec::with_capacity(6 * self.0.len());

        for peer in &self.0 {
            slice.extend(peer.ip().octets());
            slice.extend(peer.port().to_be_bytes());
        }

        serializer.serialize_bytes(&slice)
    }
}

#[allow(dead_code)]
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
            E: std::error::Error,
        {
            Ok(v.to_vec())
        }
    }

    deserializer.deserialize_bytes(VecU8Visitor)
}

#[allow(dead_code)]
fn serialize_pieces<S>(pieces: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(pieces.as_slice())
}
