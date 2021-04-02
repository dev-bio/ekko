use std::{
    
    fmt::{

        Result as FmtResult,
        Formatter, 
        Debug, 
    }, 
    
    net::{SocketAddr},
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

            Self::V4(buffer) | Self::V6(buffer) => {
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

            Self::V4(buffer) | Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(1);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("code", e.to_string())
                })?)
            }
        }
    }

    pub fn get_checksum(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => {
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
                match self.get_type()? {

                    0 => {

                        let mut cursor = Cursor::new(buffer);
                        cursor.set_position(4);
                        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("identifier", e.to_string())
                        })?)
                    }

                    _ => self.get_originator()?
                        .get_identifier()
                }
            }

            Self::V6(buffer) => {
                match self.get_type()? {

                    129 => {

                        let mut cursor = Cursor::new(buffer);
                        cursor.set_position(4);
                        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("identifier", e.to_string())
                        })?)
                    }

                    _ => self.get_originator()?
                        .get_identifier()
                }
            }
        }
    }

    pub fn get_sequence(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buffer) => {
                match self.get_type()? {

                    0 => {

                        let mut cursor = Cursor::new(buffer);
                        cursor.set_position(6);
                        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("sequence number", e.to_string())
                        })?)
                    }

                    _ => self.get_originator()?
                        .get_sequence()
                }
            }

            Self::V6(buffer) => {
                match self.get_type()? {

                    129 => {

                        let mut cursor = Cursor::new(buffer);
                        cursor.set_position(6);
                        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("sequence number", e.to_string())
                        })?)
                    }

                    _ => self.get_originator()?
                        .get_sequence()
                }
            }
        }
    }

    pub fn get_originator(&self) -> Result<EchoRequest<'a>, EkkoError> {
        match self {

            Self::V4(buffer) => {
                match self.get_type()? {

                    3 | 4 | 5 | 11 | 12 => (),

                    x => return Err({
                        EkkoError::RequestReadField("originator", {
                            format!("missing originator for type: {}", x)
                        })
                    })
                }

                let mut cursor = Cursor::new(buffer);
                cursor.set_position(8);

                let header_octets = ((cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                })? & 0x0F) * 4) as usize;

                Ok(EchoRequest::V4({
                    &(buffer[(8 + header_octets)..])
                }))
            }

            Self::V6(buffer) => {
                match self.get_type()? {

                    1 | 2 | 3 | 4 => (),

                    x => return Err({
                        EkkoError::RequestReadField("originator", {
                            format!("missing originator for type: {}", x)
                        })
                    })
                }

                Ok(EchoRequest::V6({
                    &(buffer[48..])
                }))
            }
        }
    }
}

impl<'a> Debug for EchoResponse<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        fmt.debug_struct("EchoResponse")
            .field("identifier", &(self.get_identifier()))
            .field("sequence", &(self.get_sequence()))
            .field("checksum", &(self.get_checksum()))
            .field("type", &(self.get_type()))
            .field("code", &(self.get_code()))
            .finish()
    }
}

pub(crate) enum EchoRequest<'a> {
    V4(&'a [u8]),
    V6(&'a [u8]),
}

impl<'a> EchoRequest<'a> {
    pub fn new(buffer: &'a mut [u8], idf: u16, seq: u16, src: SocketAddr, dst: SocketAddr) -> Result<EchoRequest, EkkoError> {
        match (src, dst) {

            (SocketAddr::V4(_), SocketAddr::V4(_)) => EchoRequest::new_ipv4(buffer, idf, seq),
            (SocketAddr::V6(src), SocketAddr::V6(dst)) => EchoRequest::new_ipv6(buffer, idf, seq, &(src.ip().segments()), &(dst.ip().segments())),
            _ => Err(EkkoError::RequestIpMismatch { 
                src: src.to_string(), 
                dst: dst.to_string() 
            })
        }
    }

    fn new_ipv4(buffer: &'a mut [u8], idf: u16, seq: u16) -> Result<EchoRequest, EkkoError> {
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
        
        Ok(EchoRequest::V4({
            cursor.into_inner()
        }))
    }

    fn new_ipv6<'b>(buffer: &'a mut [u8], idf: u16, seq: u16, src: &'b [u16; 8], dst: &'b [u16; 8]) -> Result<EchoRequest<'a>, EkkoError> {
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
        
        Ok(EchoRequest::V6({
            cursor.into_inner()
        }))
    }

    pub fn as_slice(&self) -> &'a [u8] {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => buffer
        }
    }

    pub fn get_type(&self) -> Result<u8, EkkoError> {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(0);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::RequestReadField("type", e.to_string())
                })?)
            }
        }
    }

    pub fn get_code(&self) -> Result<u8, EkkoError> {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(1);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::RequestReadField("code", e.to_string())
                })?)
            }
        }
    }

    pub fn get_checksum(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(2);
                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::RequestReadField("checksum", e.to_string())
                })?)
            }
        }
    }

    pub fn get_identifier(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(4);
                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::RequestReadField("identifier", e.to_string())
                })?)
            }
        }
    }

    pub fn get_sequence(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buffer) | Self::V6(buffer) => {
                let mut cursor = Cursor::new(buffer);
                cursor.set_position(6);
                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::RequestReadField("sequence", e.to_string())
                })?)
            }
        }
    }
}

impl<'a> Debug for EchoRequest<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        fmt.debug_struct("EchoRequest")
            .field("identifier", &(self.get_identifier()))
            .field("sequence", &(self.get_sequence()))
            .field("checksum", &(self.get_checksum()))
            .field("type", &(self.get_type()))
            .field("code", &(self.get_code()))
            .finish()
    }
}
