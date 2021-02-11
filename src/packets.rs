use std::{ 
    
    fmt::{
        Result as FmtResult,
        Formatter, 
        Debug, 
    },

    io::{Cursor}, 
    
};

use super::error::{EkkoError};

use byteorder::{
    WriteBytesExt,
    ReadBytesExt, 
    BigEndian, 
};

pub(crate) enum EchoResponse<'a> {
    V4(&'a [u8]),
    V6(&'a [u8]),
}

impl<'a> EchoResponse<'a> {
    pub fn get_type(&self) -> Result<u8, EkkoError> {
        match self {
            Self::V4(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(0);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("type", e.to_string())
                })?)
            },
            Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(0);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("type", e.to_string())
                })?)
            }
        }
    }

    pub fn get_code(&self) -> Result<u8, EkkoError> {
        match self {
            Self::V4(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(1);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("code", e.to_string())
                })?)
            },
            Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(1);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("code", e.to_string())
                })?)
            },
        }
    }

    pub fn get_checksum(&self) -> Result<u16, EkkoError> {
        match self {
            Self::V4(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(2);
                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("checksum", e.to_string())
                })?)
            },
            Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(2);
                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("checksum", e.to_string())
                })?)
            }
        }
    }

    pub fn get_identifier(&self) -> Result<u16, EkkoError> {
        match self {
            Self::V4(buffer) => {
                let mut cursor = Cursor::new(buffer);
                match self.get_type()? {
                    3 | 5 | 11 => {
                        cursor.set_position(8);
                        let header_octets = ((cursor.read_u8().map_err(|e| {
                            EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                        })? & 0x0F) * 4) as u64;
                        cursor.set_position((8 + header_octets) + 4)
                    }

                    _ => cursor.set_position(4)
                }

                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("identifier", e.to_string())
                })?)
            },
            Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                match self.get_type()? {
                    3 | 5 | 11 => cursor.set_position(54),
                    _ => cursor.set_position(4),
                }

                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("identifier", e.to_string())
                })?)
            },
        }
    }

    pub fn get_sequence_number(&self) -> Result<u16, EkkoError> {
        match self {
            Self::V4(buffer) => {
                let mut cursor = Cursor::new(buffer);
                match self.get_type()? {
                    3 | 5 | 11 => {
                        cursor.set_position(8);
                        let header_octets = ((cursor.read_u8().map_err(|e| {
                            EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                        })? & 0x0F) * 4) as u64;
                        cursor.set_position((8 + header_octets) + 6)
                    }

                    _ => cursor.set_position(6)
                }

                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("sequence number", e.to_string())
                })?)
            },
            Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                match self.get_type()? { 
                    3 | 5 | 11 => cursor.set_position(56),
                    _ => cursor.set_position(6),
                }

                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("sequence number", e.to_string())
                })?)
            },
        }
    }
}

impl<'a> Debug for EchoResponse<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        fmt.debug_struct("EchoResponse")
            .field("identifier", &(self.get_identifier()))
            .field("sequence", &(self.get_sequence_number()))
            .field("checksum", &(self.get_checksum()))
            .field("type", &(self.get_type()))
            .field("code", &(self.get_code()))
            .finish()
    }
}

pub(crate) struct EchoRequest<'a> {
    buffer: &'a [u8]
}

impl<'a> EchoRequest<'a> {
    pub fn new_ipv4(buffer: &'a mut [u8], idf: u16, seq: u16) -> Result<EchoRequest, EkkoError> {
        let mut cursor = Cursor::new(buffer);

        fn checksum_v4(data: &[u8]) -> u16 {
            let mut sum: u32 = data.chunks(2).map(|chunk| match chunk {
                &[a, b, ..] => u16::from_be_bytes([a, b]) as u32,
                &[.., a] => ((a as u32) << 8),
                &[..] => 0 as u32,
            }).sum();
        
            while sum >> 16 != 0 {
                sum = (sum >> 16) + (sum & 0xFFFF);
            }
        
            !(sum as u16)
        }

        cursor.write_u8(8).map_err(|e| { 
            EkkoError::RequestWriteIcmpv4Field("type", e.to_string())
        })?;

        cursor.write_u8(0).map_err(|e| { 
            EkkoError::RequestWriteIcmpv4Field("code", e.to_string())
        })?;

        cursor.write_u16::<BigEndian>(0).map_err(|e| { 
            EkkoError::RequestWriteIcmpv4Field("checksum placeholder", e.to_string())
        })?;

        cursor.write_u16::<BigEndian>(idf).map_err(|e| { 
            EkkoError::RequestWriteIcmpv4Field("identifier", e.to_string())
        })?;

        cursor.write_u16::<BigEndian>(seq).map_err(|e| { 
            EkkoError::RequestWriteIcmpv4Field("sequence", e.to_string())
        })?;

        for _ in 0..36 {
            cursor.write_u8(rand::random()).map_err(|e| {
                EkkoError::RequestWriteIcmpv4Payload(e.to_string())
            })?;
        }

        cursor.set_position(2);
        cursor.write_u16::<BigEndian>(checksum_v4(cursor.get_ref())).map_err(|e| {
            EkkoError::RequestWriteIcmpv4Field("checksum", e.to_string())
        })?;
        
        Ok(EchoRequest {
            buffer: cursor.into_inner()
        })
    }

    pub fn new_ipv6<'b>(buffer: &'a mut [u8], idf: u16, seq: u16, src: &'b [u16; 8], dst: &'b [u16; 8]) -> Result<EchoRequest<'a>, EkkoError> {
        let mut cursor = Cursor::new(buffer);

        fn checksum_v6(data: &[u8], src: &[u16; 8], dst: &[u16; 8]) -> u16 {
            fn sum_segments(segments: &[u16; 8]) -> u32 {
                segments.iter().fold(0, |n, x| {
                    n + (x.clone() as u32)
                })
            }
        
            let mut sum: u32 = data.chunks(2).map(|chunk| match chunk {
                &[a, b, ..] => u16::from_be_bytes([a, b]) as u32,
                &[.., a] => ((a as u32) << 8),
                &[..] => 0 as u32,
            }).sum();
            
            sum += sum_segments(src);
            sum += sum_segments(dst);
            sum += (data.len() + 16) as u32;
            sum += 58;
        
            while sum >> 16 != 0 {
                sum = (sum >> 16) + (sum & 0xFFFF);
            }
        
            !(sum as u16)
        }

        cursor.write_u8(128).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("type", e.to_string())
        })?;

        cursor.write_u8(0).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("code", e.to_string())
        })?;

        cursor.write_u16::<BigEndian>(0xFFFF).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("checksum placeholder", e.to_string())
        })?;

        cursor.write_u16::<BigEndian>(idf).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("identifier", e.to_string())
        })?;

        cursor.write_u16::<BigEndian>(seq).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("sequence", e.to_string())
        })?;

        for _ in 0..16 {
            cursor.write_u8(rand::random()).map_err(|e| {
                EkkoError::RequestWriteIcmpv6Payload(e.to_string())
            })?;
        }

        cursor.set_position(2);
        cursor.write_u16::<BigEndian>(checksum_v6(cursor.get_ref(), src, dst)).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("checksum", e.to_string())
        })?;
        
        Ok(EchoRequest {
            buffer: cursor.into_inner()
        })
    }

    pub fn as_slice(&self) -> &'a [u8] {
        self.buffer
    }

    pub fn get_type(&self) -> Result<u8, EkkoError> {
        let mut cursor = Cursor::new(self.buffer);
        cursor.set_position(0);
        Ok(cursor.read_u8().map_err(|e| {
            EkkoError::RequestReadField("type", e.to_string())
        })?)
    }

    pub fn get_code(&self) -> Result<u8, EkkoError> {
        let mut cursor = Cursor::new(self.buffer);
        cursor.set_position(1);
        Ok(cursor.read_u8().map_err(|e| {
            EkkoError::RequestReadField("code", e.to_string())
        })?)
    }

    pub fn get_checksum(&self) -> Result<u16, EkkoError> {
        let mut cursor = Cursor::new(self.buffer);
        cursor.set_position(2);
        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
            EkkoError::RequestReadField("checksum", e.to_string())
        })?)
    }

    pub fn get_identifier(&self) -> Result<u16, EkkoError> {
        let mut cursor = Cursor::new(self.buffer);
        cursor.set_position(4);
        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
            EkkoError::RequestReadField("identifier", e.to_string())
        })?)
    }

    pub fn get_sequence_number(&self) -> Result<u16, EkkoError> {
        let mut cursor = Cursor::new(self.buffer);
        cursor.set_position(6);
        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
            EkkoError::RequestReadField("sequence", e.to_string())
        })?)
    }
}

impl<'a> Debug for EchoRequest<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        fmt.debug_struct("EchoRequest")
            .field("identifier", &(self.get_identifier()))
            .field("sequence", &(self.get_sequence_number()))
            .field("checksum", &(self.get_checksum()))
            .field("type", &(self.get_type()))
            .field("code", &(self.get_code()))
            .finish()
    }
}
