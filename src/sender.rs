use std::{
    
    ops::{Range}, 
    io::{Cursor},

    time::{

        Duration, 
        Instant,
    },

    net::{

        SocketAddrV6, 
        SocketAddrV4, 
        SocketAddr, 
    }, 
    
    net::{

        Ipv6Addr, 
        Ipv4Addr, 
        IpAddr, 
    },   
};

use byteorder::{ReadBytesExt};

use socket2::{

    Protocol, 
    Domain, 
    Socket, 
    Type,
};

use super::{

    packets::{EkkoPacket},
    error::{EkkoError},

    responses::{
        
        EkkoResponse,
        EkkoData,
    },
};

/// Take a look at the default implementation.
pub struct EkkoSettings {
    
    pub identifier: u16,
    pub sequence: u16,
    
    pub timeout: Duration,
}

impl Default for EkkoSettings {
    fn default() -> EkkoSettings {
        EkkoSettings {

            identifier: rand::random(),
            sequence: 0,

            timeout: {

                Duration::from_millis(1000)
            },
        }
    }
}

pub struct Ekko {

    source_socket_address: SocketAddr,
    target_socket_address: SocketAddr,

    socket: Socket,
}

impl Ekko {

    /// Build a sender with given target address.
    pub fn with_target<T: Into<IpAddr>>(target: T) -> Result<Ekko, EkkoError> {
        match target.into() {

            IpAddr::V4(target) => {

                let source_address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
                let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).map_err(|e| {
                    EkkoError::SocketCreateIcmpv4(e.to_string())
                })?;

                socket.set_nonblocking(true).map_err(|e| {
                    EkkoError::SocketSetNonBlocking(true, e.to_string())
                })?;

                socket.set_recv_buffer_size(256).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|e| {
                    EkkoError::SocketBindIpv4(source_address.to_string(), e.to_string())
                })?;
                
                Ok(Ekko {

                    source_socket_address: SocketAddr::V4(source_address),
                    target_socket_address: SocketAddr::V4({
                        SocketAddrV4::new(target, 0)
                    }),

                    socket: socket,
                })
            }

            IpAddr::V6(target) => {

                let source_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0, 0, 0);
                let socket = Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6)).map_err(|e| {
                    EkkoError::SocketCreateIcmpv6(e.to_string())
                })?;

                socket.set_nonblocking(true).map_err(|e| {
                    EkkoError::SocketSetNonBlocking(true, e.to_string())
                })?;

                socket.set_recv_buffer_size(512).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|e| {
                    EkkoError::SocketBindIpv6(source_address.to_string(), e.to_string())
                })?;
  
                Ok(Ekko {

                    source_socket_address: SocketAddr::V6(source_address),
                    target_socket_address: SocketAddr::V6({
                        SocketAddrV6::new(target, 0, 0, 0)
                    }),

                    socket: socket,
                })
            }
        }
    }

    /// Send an echo request with default settings.
    pub fn send(&self, hops: u32) -> Result<EkkoResponse, EkkoError> {
        self.send_with_settings(hops, Default::default())
    }

    /// Send an echo request with user defined settings.
    pub fn send_with_settings(&self, hops: u32, EkkoSettings { 
        timeout, identifier, sequence 
    }: EkkoSettings) -> Result<EkkoResponse, EkkoError> {

        let mut buf: [u8; 256] = {
            [0; 256]
        };

        let timepoint = Instant::now();

        self.inner_send(hops, {
            (identifier, sequence)
        })?;

        let result = loop {

            let identifier = identifier;
            if let Some((address, packet)) = self.inner_recv(&mut buf)? {
                if identifier == packet.get_identifier()? {
                    if sequence == packet.get_sequence()? {
                        let time = (timepoint.clone(), timepoint.elapsed());
                        let net = (address.clone(), hops.clone());

                        break EkkoResponse::new(net, time, packet)?;
                    }
                }
            }

            if timepoint.elapsed() < timeout {
                continue
            }

            break EkkoResponse::Lacking({

                EkkoData { 

                    timepoint: timepoint, 
                    elapsed: timepoint
                        .elapsed(),

                    address: None,
                    
                    identifier: identifier,
                    sequence: sequence,
                    hops: hops,
                    
                }
            })
        };

        Ok(result)
    }

    /// Send echo requests for all hops in range with default settings.
    pub fn send_range(&self, hops: Range<u32>) -> Result<Vec<EkkoResponse>, EkkoError> {
        self.send_range_with_settings(hops, Default::default())
    }

    /// Send echo requests for all hops in range with user defined settings.
    pub fn send_range_with_settings(&self, hops: Range<u32>, EkkoSettings { 
        timeout, identifier, mut sequence 
    }: EkkoSettings) -> Result<Vec<EkkoResponse>, EkkoError> {

        let mut buf: [u8; 256] = {
            [0; 256]
        };
        
        let mut echo_responses = Vec::with_capacity(hops.len());
        let mut echo_requests = Vec::with_capacity(hops.len());
        let mut echo_route = Vec::with_capacity(hops.len());

        let timepoint = Instant::now();
        
        for hop in hops {
            
            self.inner_send(hop, {
                (identifier, sequence)
            })?;
            
            echo_requests.push((timepoint.clone(), identifier.clone(), sequence.clone(), hop.clone()));
            sequence = sequence.wrapping_add(1);
        }
        
        loop {

            if let Some((address, response)) = self.inner_recv(&mut buf)? {
                for (request_timepoint, request_identifier, request_sequence, _) 
                    in echo_requests.iter() {

                    match (request_identifier.clone(), request_sequence.clone()) {

                        x if x == (response.get_identifier()?, response.get_sequence()?) => echo_responses.push({
                            (address.clone(), request_timepoint.clone(), request_timepoint.elapsed(), buf.to_vec())
                        }),
                        
                        _ => continue
                    }

                    break
                }
            }

            if (echo_requests.len() - echo_responses.len()) > 0 {
                if timepoint.elapsed() < timeout {
                    continue
                }
            }
            
            for (request_timepoint, request_identifier, request_sequence, request_hops) 
                in echo_requests.iter() {

                let previous_route_length = echo_route.len();
                for (response_address, response_timepoint, response_elapsed, buf) 
                    in echo_responses.iter() {

                    let packet = match response_address {
                        IpAddr::V4(_) => {

                            let mut cursor = Cursor::new(buf.as_slice());
                            let header_octets = ((cursor.read_u8().map_err(|e| {
                                EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                            })? & 0x0F) * 4) as usize;

                            EkkoPacket::V4(&(buf[header_octets..]))
                        },

                        IpAddr::V6(_) => {

                            EkkoPacket::V6(&(buf[..]))
                        },
                    };

                    match (request_identifier.clone(), request_sequence.clone()) {

                        x if x == (packet.get_identifier()?, packet.get_sequence()?) => echo_route.push({
                            let time = (response_timepoint.clone(), response_elapsed.clone());
                            let net = (response_address.clone(), request_hops.clone());
    
                            EkkoResponse::new(net, time, packet)?
                        }),

                        _ => continue
                    }

                    break
                }

                if previous_route_length < echo_route.len() { 
                    continue 
                } 
                
                else {

                    echo_route.push({

                        EkkoResponse::Lacking({

                            EkkoData { 

                                timepoint: request_timepoint.clone(), 
                                elapsed: request_timepoint
                                    .elapsed(),

                                address: None,

                                identifier: request_identifier.clone(),
                                sequence: request_sequence.clone(),
                                hops: request_hops.clone(),

                            }
                        })
                    });
                }
            }

            break Ok(echo_route)
        }
    }

    fn inner_send(&self, hops: u32, pkt: (u16, u16)) -> Result<(), EkkoError> {
        let mut buf: [u8; 128] = [0; 128];
        let request = EkkoPacket::new(&mut buf[..], pkt, {
            (self.source_socket_address, self.target_socket_address)
        })?;

        match (self.source_socket_address, self.target_socket_address) {

            (SocketAddr::V4(_), SocketAddr::V4(_)) => {
                self.socket.set_ttl(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv4({
                        e.to_string()
                    })
                })?;

                self.socket.send_to(request.as_slice(), {
                    &(self.target_socket_address.into())
                }).map_err(|e|  EkkoError::SocketSendIcmpv4({
                    e.to_string()
                }))?;
            },

            (SocketAddr::V6(_), SocketAddr::V6(_)) => {
                self.socket.set_unicast_hops_v6(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv6({
                        e.to_string()
                    })
                })?;

                self.socket.send_to(request.as_slice(), {
                    &(self.target_socket_address.into())
                }).map_err(|e| EkkoError::SocketSendIcmpv6({
                    e.to_string()
                }))?;
            },

            (src, dst) => {
                return Err(EkkoError::SocketIpMismatch { 
                    src: src.to_string(), 
                    dst: dst.to_string() 
                })
            },
        };

        Ok(())
    }

    fn inner_recv<'a>(&self, buf: &'a mut [u8]) -> Result<Option<(IpAddr, EkkoPacket<'a>)>, EkkoError> {
        let result = self.socket.recv_from(unsafe { 
            std::mem::transmute(&mut buf[..]) 
        });

        if let Ok((_, responder)) = result {
            let responding_address = match self.source_socket_address {

                SocketAddr::V4(_) => IpAddr::V4(responder.as_socket_ipv4()
                    .ok_or(EkkoError::SocketReceiveNoIpv4)?.ip()
                    .clone()),
                    
                SocketAddr::V6(_) => IpAddr::V6(responder.as_socket_ipv6()
                    .ok_or(EkkoError::SocketReceiveNoIpv6)?.ip()
                    .clone()),
            };

            match responding_address {

                IpAddr::V4(_) => Ok(Some((responding_address, {
                    let mut cursor = Cursor::new(&mut buf[..]);
                    let header_octets = ((cursor.read_u8().map_err(|e| {
                        EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                    })? & 0x0F) * 4) as usize;

                    EkkoPacket::V4(&(buf[header_octets..]))
                }))),

                IpAddr::V6(_) => Ok(Some((responding_address, {
                    EkkoPacket::V6(&(buf[..]))
                }))),
            }
        }
        
        else {
            
            Ok(None)
        }
    }
}
