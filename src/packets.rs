use std::{
    
    fmt::{

        Result as FmtResult,
        Formatter, 
        Debug, 
    }, 
    
    io::{

        Cursor, 
        Read,
    }, 
    
    net::{

        SocketAddr,
        Ipv4Addr, 
    },
};

use crate::{

    UnreachableCodeV6,
    UnreachableCodeV4,
    Unreachable,
    Redirect,
};

use super::error::{EkkoError};

use byteorder::{
    
    WriteBytesExt,
    ReadBytesExt, 
    BigEndian, 
};

pub(crate) enum EkkoPacket<'a> {
    V4(&'a [u8]),
    V6(&'a [u8]),
}

impl<'a> EkkoPacket<'a> {
    pub fn new(buf: &'a mut [u8], pkt: (u16, u16), net: (SocketAddr, SocketAddr)) -> Result<EkkoPacket, EkkoError> {
        match net {

            (SocketAddr::V4(_), SocketAddr::V4(_)) => EkkoPacket::new_ipv4(buf, pkt),
            (SocketAddr::V6(src), SocketAddr::V6(dst)) => EkkoPacket::new_ipv6(buf, pkt, (src.ip().segments(), dst.ip().segments())),
            (src, dst) => Err(EkkoError::RequestIpMismatch { 
                src: src.to_string(), 
                dst: dst.to_string() 
            })
        }
    }

    fn new_ipv4(buf: &'a mut [u8], pkt: (u16, u16)) -> Result<EkkoPacket, EkkoError> {
        let (idf, seq) = pkt;

        fn checksum_v4(data: &[u8]) -> u16 {
            let mut sum: u32 = data.chunks(2).map(|chunk| match chunk {
                
                &[ .. , a, b ] => u16::from_be_bytes([a, b]) as u32,
                &[ .. , a] => u16::from_be_bytes([a, 0]) as u32,
                &[ .. ] => 0 as u32,
                
            }).sum();
            
            while sum >> 16 != 0 {
                sum = (sum >> 16) + (sum & 0xFFFF);
            }
            
            !(sum as u16)
        }
        
        let mut cursor = Cursor::new(buf);

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
        
        Ok(EkkoPacket::V4({
            cursor.into_inner()
        }))
    }
    
    fn new_ipv6<'b>(buf: &'a mut [u8], pkt: (u16, u16), net: ([u16; 8], [u16; 8])) -> Result<EkkoPacket<'a>, EkkoError> {
        let (idf, seq) = pkt;

        fn checksum_v6(data: &[u8], net: ([u16; 8], [u16; 8])) -> u16 {
            let (src, dst) = net;

            fn sum_segments(segments: [u16; 8]) -> u32 {
                segments.iter().fold(0, |n, x| {
                    n + (x.clone() as u32)
                })
            }
            
            let mut sum: u32 = data.chunks(2).map(|chunk| match chunk {
                
                &[ .. , a, b ] => u16::from_be_bytes([a, b]) as u32,
                &[ .. , a] => u16::from_be_bytes([a, 0]) as u32,
                &[ .. ] => 0 as u32,
                
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

        let mut cursor = Cursor::new(buf);

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
        cursor.write_u16::<BigEndian>(checksum_v6(cursor.get_ref(), net)).map_err(|e| {
            EkkoError::RequestWriteIcmpv6Field("checksum", e.to_string())
        })?;
        
        Ok(EkkoPacket::V6({
            cursor.into_inner()
        }))
    }

    pub fn as_slice(&self) -> &'a [u8] {
        match self {

            Self::V4(buf) | Self::V6(buf) => buf
        }
    }

    pub fn get_type(&self) -> Result<u8, EkkoError> {
        match self {

            Self::V4(buf) | Self::V6(buf) => {
                let mut cursor = Cursor::new(buf);
                cursor.set_position(0);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("type", e.to_string())
                })?)
            }
        }
    }

    pub fn get_code(&self) -> Result<u8, EkkoError> {
        match self {

            Self::V4(buf) | Self::V6(buf) => {
                let mut cursor = Cursor::new(buf);
                cursor.set_position(1);
                Ok(cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("code", e.to_string())
                })?)
            }
        }
    }

    pub fn get_checksum(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buf) | Self::V6(buf) => {
                let mut cursor = Cursor::new(buf);
                cursor.set_position(2);
                Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                    EkkoError::ResponseReadField("checksum", e.to_string())
                })?)
            }
        }
    }

    pub fn get_identifier(&self) -> Result<u16, EkkoError> {
        match self {

            Self::V4(buf) => {
                match self.get_type()? {

                    8 | 0 => {

                        let mut cursor = Cursor::new(buf);
                        cursor.set_position(4);
                        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("identifier", e.to_string())
                        })?)
                    }

                    _ => self.get_originator()?
                        .get_identifier()
                }
            }

            Self::V6(buf) => {
                match self.get_type()? {

                    128 | 129 => {

                        let mut cursor = Cursor::new(buf);
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

            Self::V4(buf) => {
                match self.get_type()? {

                    8 | 0 => {

                        let mut cursor = Cursor::new(buf);
                        cursor.set_position(6);
                        Ok(cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("sequence number", e.to_string())
                        })?)
                    }

                    _ => self.get_originator()?
                        .get_sequence()
                }
            }

            Self::V6(buf) => {
                match self.get_type()? {

                    128 | 129 => {

                        let mut cursor = Cursor::new(buf);
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

    pub fn get_originator(&self) -> Result<EkkoPacket<'a>, EkkoError> {
        match self {

            Self::V4(buf) => {
                match self.get_type()? {

                    3 | 4 | 5 | 11 | 12 => {

                        let mut cursor = Cursor::new(buf);
                        cursor.set_position(8);

                        let header_octets = ((cursor.read_u8().map_err(|e| {
                            EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                        })? & 0x0F) * 4) as usize;

                        Ok(EkkoPacket::V4({
                            &(buf[(8 + header_octets)..])
                        }))
                    },

                    x => Err({
                        EkkoError::RequestReadField("originator", {
                            format!("missing originator for type: {}", x)
                        })
                    })
                }
            }

            Self::V6(buf) => {
                match self.get_type()? {

                    1 | 2 | 3 | 4 => (),

                    x => return Err({
                        EkkoError::RequestReadField("originator", {
                            format!("missing originator for type: {}", x)
                        })
                    })
                }

                Ok(EkkoPacket::V6({
                    &(buf[48..])
                }))
            }
        }
    }

    pub fn get_redirect(&self) -> Result<Redirect, EkkoError> {
        match self {

            Self::V4(buf) => {
                match self.get_type()? {

                    5 => {

                        let mut cursor = Cursor::new(buf);
                        let mut octets: [u8; 4] = [0; 4];

                        cursor.set_position(4);
                        cursor.read_exact(&mut octets).map_err(|e| {
                            EkkoError::ResponseReadField("address octet: 0", e.to_string())
                        })?;

                        Ok(match self.get_code()? {
                            0 => Redirect::RedirectDatagramsForNetwork(Ipv4Addr::from(octets)),
                            1 => Redirect::RedirectDatagramsForHost(Ipv4Addr::from(octets)),
                            2 => Redirect::RedirectDatagramsForTypeServiceNetwork(Ipv4Addr::from(octets)),
                            3 => Redirect::RedirectDatagramsForTypeServiceHost(Ipv4Addr::from(octets)),
                            code => Redirect::Unexpected(code),
                        })
                    }

                    _ => Err(EkkoError::RequestReadIcmpv4Type("redirect", {
                        "not redirect response".to_owned()
                    }))
                }
            }

            Self::V6(_) => {
                match self.get_type()? {

                    _ => Err(EkkoError::RequestReadIcmpv6Type("redirect", {
                        "not a redirect response".to_owned()
                    }))
                }
            }
        }
    }

    pub fn get_unreachable(&self) -> Result<Unreachable, EkkoError> {
        match self {

            Self::V4(buf) => {
                match self.get_type()? {

                    3 => {

                        let mut cursor = Cursor::new(buf);
                        cursor.set_position(6);
                        let value = cursor.read_u16::<BigEndian>().map_err(|e| {
                            EkkoError::ResponseReadField("problem pointer", e.to_string())
                        })?;

                        Ok(Unreachable::V4(match self.get_code()? {
                            0  => UnreachableCodeV4::DestinationNetworkUnreachable(value),
                            1  => UnreachableCodeV4::DestinationHostUnreachable(value),
                            2  => UnreachableCodeV4::DestinationProtocolUnreachable(value),
                            3  => UnreachableCodeV4::DestinationPortUnreachable(value),
                            4  => UnreachableCodeV4::FragmentationRequired(value),
                            5  => UnreachableCodeV4::SourceRouteFailed(value),
                            6  => UnreachableCodeV4::DestinationNetworkUnknown(value),
                            7  => UnreachableCodeV4::DestinationHostUnknown(value),
                            8  => UnreachableCodeV4::SourceHostIsolated(value),
                            9  => UnreachableCodeV4::NetworkAdministrativelyProhibited(value),
                            10 => UnreachableCodeV4::HostAdministrativelyProhibited(value),
                            11 => UnreachableCodeV4::NetworkUnreachable(value),
                            12 => UnreachableCodeV4::HostUnreachable(value),
                            13 => UnreachableCodeV4::CommunicationAdministrativelyProhibited(value),
                            14 => UnreachableCodeV4::HostPrecedenceViolation(value),
                            15 => UnreachableCodeV4::PrecedenceCutoff(value),
                            code => UnreachableCodeV4::Unexpected(code),
                        }))
                    }

                    _ => Err(EkkoError::RequestReadIcmpv4Type("unreachable", {
                        "not an unreachable response".to_owned()
                    }))
                }
            }

            Self::V6(_) => {
                match self.get_type()? {

                    1 => {

                        Ok(Unreachable::V6(match self.get_code()? {
                            0 => UnreachableCodeV6::NoRouteToDestination,
                            1 => UnreachableCodeV6::CommunicationWithDestinationAdministrativelyProhibited,
                            2 => UnreachableCodeV6::BeyondScopeOfSourceAddress,
                            3 => UnreachableCodeV6::AddressUnreachable,
                            4 => UnreachableCodeV6::PortUnreachable,
                            5 => UnreachableCodeV6::SourceAddressFailedIngressEgressPolicy,
                            6 => UnreachableCodeV6::RejectRouteToDestination,
                            7 => UnreachableCodeV6::ErrorInSourceRoutingHeader,
                            code => UnreachableCodeV6::Unexpected(code),
                        }))
                    }

                    _ => Err(EkkoError::RequestReadIcmpv6Type("unreachable", {
                        "not an unreachable response".to_owned()
                    }))
                }
            }
        }
    }
}

impl<'a> Debug for EkkoPacket<'a> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        fmt.debug_struct("EkkoPacket")
            .field("identifier", &(self.get_identifier()))
            .field("sequence", &(self.get_sequence()))
            .field("checksum", &(self.get_checksum()))
            .field("type", &(self.get_type()))
            .field("code", &(self.get_code()))
            .finish()
    }
}