use std::{

    net::{
        ToSocketAddrs,
        SocketAddrV6, 
        SocketAddrV4, 
        SocketAddr, 
    }, 
    
    net::{
        Ipv6Addr, 
        Ipv4Addr, 
        IpAddr, 
    }, 
    
    time::{
        Duration, 
        Instant,
    },

    io::{Cursor}, 
};

use byteorder::{ReadBytesExt};

use socket2::{
    Protocol, 
    Domain, 
    Socket, 
    Type,
};

use super::{

    error::{EkkoError},

    packets::{
        EchoResponse,
        EchoRequest, 
    },

    responses::{
        EkkoResponse,
        EkkoData,
        Unreachable,
        UnreachableCodeV6,
        UnreachableCodeV4,
    },
};

pub struct Ekko {
    source_socket_address: SocketAddr,
    target_socket_address: SocketAddr,
    sequence_number: u16,
    identifier: u16,
    buffer_receive: Vec<u8>,
    buffer_send: Vec<u8>,
    socket: Socket,
}

impl Ekko {
    /// Send an echo request with a default timeout of 100 milliseconds ..
    pub fn send(&mut self, hops: u32) -> Result<EkkoResponse, EkkoError> {
        self.send_with_timeout(hops, Duration::from_millis(100))
    }

    /// Send an echo request with or without a timeout ..
    pub fn send_with_timeout(&mut self, hops: u32, timeout: Duration) -> Result<EkkoResponse, EkkoError> {
        let sequence_number = self.sequence_number;
        let identifier = self.identifier;
        
        let timepoint = Instant::now();
        match (self.source_socket_address, self.target_socket_address) {
            (SocketAddr::V6(source), SocketAddr::V6(target)) => {
                self.socket.set_unicast_hops_v6(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv6(e.to_string())
                })?;

                self.buffer_send.resize(64, 0);
                let request = EchoRequest::new_ipv6(self.buffer_send.as_mut_slice(), identifier, sequence_number, &(source.ip().segments()), &(target.ip().segments()))?;
                self.socket.send_to(request.as_slice(), &(target.into())).map_err(|e| {
                    EkkoError::SocketSend(e.to_string())
                })?;
            },
            (SocketAddr::V4(_), SocketAddr::V4(target)) => {
                self.socket.set_ttl(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv4(e.to_string())
                })?;

                self.buffer_send.resize(64, 0);
                let request = EchoRequest::new_ipv4(self.buffer_send.as_mut_slice(), identifier, sequence_number)?;
                self.socket.send_to(request.as_slice(), &(target.into())).map_err(|e| {
                    EkkoError::SocketSend(e.to_string())
                })?;
            },
            (SocketAddr::V4(source), SocketAddr::V6(target)) => {
                return Err(EkkoError::SocketIpMismatch { 
                    src: source.to_string(), 
                    tgt: target.to_string() 
                })
            },
            (SocketAddr::V6(source), SocketAddr::V4(target)) => {
                return Err(EkkoError::SocketIpMismatch { 
                    src: source.to_string(), 
                    tgt: target.to_string() 
                })
            },
        };

        self.sequence_number = {
            self.sequence_number.wrapping_add(1)
        };

        loop {
            if timepoint.elapsed() > timeout {
                break Ok(EkkoResponse::LackingResponse(EkkoData { 
                    timepoint: timepoint, 
                    elapsed: timepoint.elapsed(),
                    address: None,
                    hops: hops,
                }))
            }

            self.socket.set_read_timeout(Some(timeout - timepoint.elapsed())).map_err(|e| {
                EkkoError::SocketSetReadTimeout(e.to_string())
            })?;

            self.buffer_receive.resize(512, 0);
            if let Ok((_, responder)) = self.socket.recv_from(self.buffer_receive.as_mut_slice()) {
                let responding_address = match self.source_socket_address {
                    SocketAddr::V6(_) => IpAddr::V6(responder.as_inet6()
                        .ok_or(EkkoError::SocketReceiveNoIpv6)?.ip().clone()),
                    SocketAddr::V4(_) => IpAddr::V4(responder.as_inet()
                        .ok_or(EkkoError::SocketReceiveNoIpv4)?.ip().clone()),
                };

                let mut cursor = Cursor::new(self.buffer_receive.as_slice());
                let header_octets = ((cursor.read_u8().map_err(|e| {
                    EkkoError::ResponseReadField("internet protocol header size", e.to_string())
                })? & 0x0F) * 4) as usize;

                let elapsed = timepoint.elapsed();
                let response = match responding_address {
                    IpAddr::V4(_) => EchoResponse::V4(&self.buffer_receive[header_octets..]),
                    IpAddr::V6(_) => EchoResponse::V6(&self.buffer_receive[..]),
                };

                if identifier != response.get_identifier()? { 
                    continue 
                }

                if sequence_number != response.get_sequence_number()? { 
                    continue 
                }

                match responding_address {
                    IpAddr::V6(_) => match response.get_type()? {
                        1 => {
                            let unreachable_code = Unreachable::V6(match response.get_code()? {
                                0 => UnreachableCodeV6::NoRouteToDestination,
                                1 => UnreachableCodeV6::CommunicationWithDestinationAdministrativelyProhibited,
                                2 => UnreachableCodeV6::BeyondScopeOfSourceAddress,
                                3 => UnreachableCodeV6::AddressUnreachable,
                                4 => UnreachableCodeV6::PortUnreachable,
                                5 => UnreachableCodeV6::SourceAddressFailedIngressEgressPolicy,
                                6 => UnreachableCodeV6::RejectRouteToDestination,
                                7 => UnreachableCodeV6::ErrorInSourceRoutingHeader,
                                _ => UnreachableCodeV6::Unexpected(response.get_code()?),
                            });

                            break Ok(EkkoResponse::UnreachableResponse((EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }, unreachable_code)))
                        },
                        3 => {
                            break Ok(EkkoResponse::ExceededResponse(EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }))
                        },
                        129 => {
                            break Ok(EkkoResponse::DestinationResponse(EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }))
                        },
                        _ => {
                            let unexpected = (response.get_type()?, response.get_code()?);

                            break Ok(EkkoResponse::UnexpectedResponse((EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }, unexpected)))
                        },
                    },
                    IpAddr::V4(_) => match response.get_type()? {
                        3 => {
                            let unreachable_code = Unreachable::V4(match response.get_code()? {
                                0  => UnreachableCodeV4::DestinationNetworkUnreachable,
                                1  => UnreachableCodeV4::DestinationHostUnreachable,
                                2  => UnreachableCodeV4::DestinationProtocolUnreachable,
                                3  => UnreachableCodeV4::DestinationPortUnreachable,
                                4  => UnreachableCodeV4::FragmentationRequired,
                                5  => UnreachableCodeV4::SourceRouteFailed,
                                6  => UnreachableCodeV4::DestinationNetworkUnknown,
                                7  => UnreachableCodeV4::DestinationHostUnknown,
                                8  => UnreachableCodeV4::SourceHostIsolated,
                                9  => UnreachableCodeV4::NetworkAdministrativelyProhibited,
                                10 => UnreachableCodeV4::HostAdministrativelyProhibited,
                                11 => UnreachableCodeV4::NetworkUnreachable,
                                12 => UnreachableCodeV4::HostUnreachable,
                                13 => UnreachableCodeV4::CommunicationAdministrativelyProhibited,
                                14 => UnreachableCodeV4::HostPrecedenceViolation,
                                15 => UnreachableCodeV4::PrecedenceCutoff,
                                _ => UnreachableCodeV4::Unexpected(response.get_code()?),
                            });

                            break Ok(EkkoResponse::UnreachableResponse((EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }, unreachable_code)))
                        },
                        11 => {
                            break Ok(EkkoResponse::ExceededResponse(EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }))
                        },
                        0 => {
                            break Ok(EkkoResponse::DestinationResponse(EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }))
                        },
                        _ => {
                            let unexpected = (response.get_type()?, response.get_code()?);

                            break Ok(EkkoResponse::UnexpectedResponse((EkkoData { 
                                timepoint: timepoint, 
                                elapsed: elapsed,
                                address: Some(responding_address),
                                hops: hops,
                            }, unexpected)))
                        },
                    },
                }
            } else {
                break Ok(EkkoResponse::LackingResponse(EkkoData { 
                    timepoint: timepoint, 
                    elapsed: timepoint.elapsed(),
                    address: None,
                    hops: hops,
                }))
            }
        }
    }

    /// Build a client with target address ..
    pub fn with_target(target: &str) -> Result<Ekko, EkkoError>  {
        let target_socket_address = target.to_socket_addrs().or_else(|_| {
            format!("{}:0", target).to_socket_addrs().map_err(|e| {
                EkkoError::BadTarget(target.to_string(), e.to_string())
            })
        }).and_then(|results| results.last().ok_or({
            EkkoError::UnresolvedTarget(target.to_string())
        })).and_then(|result| Ok(result))?;

        match target_socket_address.ip() {
            IpAddr::V4(_) => {
                let source_address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
                let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).map_err(|e| {
                    EkkoError::SocketCreateIcmpv4(e.to_string())
                })?;

                socket.set_recv_buffer_size(512).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|e| {
                    EkkoError::SocketBindIpv4(source_address.to_string(), e.to_string())
                })?;
                
                Ok(Ekko {
                    source_socket_address: SocketAddr::V4(source_address),
                    target_socket_address: target_socket_address,
                    sequence_number: 0,
                    identifier: rand::random(),
                    buffer_receive: Vec::with_capacity(512),
                    buffer_send: Vec::with_capacity(64),
                    socket: socket,
                })
            },
            IpAddr::V6(_) => {
                let source_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0, 0, 0);
                let socket = Socket::new(Domain::ipv6(), Type::raw(), Some(Protocol::icmpv6())).map_err(|e| {
                    EkkoError::SocketCreateIcmpv6(e.to_string())
                })?;

                socket.set_recv_buffer_size(512).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|e| {
                    EkkoError::SocketBindIpv6(source_address.to_string(), e.to_string())
                })?;
  
                Ok(Ekko {
                    source_socket_address: SocketAddr::V6(source_address),
                    target_socket_address: target_socket_address,
                    sequence_number: 0,
                    identifier: rand::random(),
                    buffer_receive: Vec::with_capacity(512),
                    buffer_send: Vec::with_capacity(64),
                    socket: socket,
                })
            },
        }
    }
}
