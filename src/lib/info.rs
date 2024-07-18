use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Info {
    ///Suggested name (or directory if multifile) to save as.
    pub name: String,

    ///Number of bytes in each piece the file is split into. Files are split into fixed-size lenght
    ///pieces which are all the same except for the last one, which may be truncated. It is almost
    ///always a power of two, most commonly 2 18 = 256K.
    #[serde(rename = "piece length")]
    pub piece_length: usize,

    ///String with length multiple of 20. It is subdivided into strings with length of 20, each of
    ///which is the SHA1 hash of the piece at the corresponding index.
    #[serde(
        deserialize_with = "deserialize_pieces",
        serialize_with = "serialize_pieces"
    )]
    pub pieces: Vec<u8>,

    ///Length of a single file, in number of bytes
    pub length: usize,

    ///A list of UTF-8 encoded strings corresponding to subdirectory names, the last of which is
    ///the actual file name (a zero length list is an error case)
    pub path: Option<Vec<Vec<String>>>,
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
            E: std::error::Error,
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
