use crate::chunk_type::ChunkType;
use crc::Crc;
use std::{fmt::Display, string::FromUtf8Error};

#[derive(Debug, PartialEq)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

#[derive(Debug)]
pub enum ChunkError {
    TooShort,
    LengthMismatched,
    InvalidChunkType,
    InvalidCrc,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let length = data.len() as u32;

        let mut payload = chunk_type.bytes().to_vec();
        payload.extend_from_slice(&data);

        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let crc = crc.checksum(&payload[..]);

        Chunk {
            length,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        Ok(String::from_utf8(self.data.clone())?)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = self.length.to_be_bytes().to_vec();
        vec.extend_from_slice(&self.chunk_type.bytes());
        vec.extend_from_slice(&self.data);
        vec.extend_from_slice(&self.crc.to_be_bytes());
        vec
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk {} with len {}", self.chunk_type, self.length)
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 12 {
            return Err(ChunkError::TooShort);
        }

        let mut first_4_bytes = [0u8; 4];
        first_4_bytes.copy_from_slice(&value[..4]);
        let length = u32::from_be_bytes(first_4_bytes);
        if 12 + length as usize != value.len() {
            return Err(ChunkError::LengthMismatched);
        }

        let mut next_4_bytes = [0u8; 4];
        next_4_bytes.copy_from_slice(&value[4..8]);
        let chunk_type = match ChunkType::try_from(next_4_bytes) {
            Ok(ct) => ct,
            Err(_) => return Err(ChunkError::InvalidChunkType),
        };

        let data = value[8..8 + length as usize].to_vec();

        let mut last_4_bytes = [0u8; 4];
        last_4_bytes.copy_from_slice(&value[8 + length as usize..]);
        let crc_found = u32::from_be_bytes(last_4_bytes);

        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let crc_expected = crc.checksum(&value[4..8 + length as usize]);

        if crc_found != crc_expected {
            Err(ChunkError::InvalidCrc)
        } else {
            Ok(Chunk {
                length,
                chunk_type,
                data,
                crc: crc_found,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }

    #[test]
    pub fn test_as_bytes_back_to_chunk() {
        let chunk = testing_chunk();
        let bytes = chunk.as_bytes();

        let new_chunk = Chunk::try_from(&bytes[..]).unwrap();
        assert_eq!(chunk, new_chunk);
    }
}
