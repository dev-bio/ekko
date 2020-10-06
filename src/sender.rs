use std::{

    collections::{HashMap}, 

    net::{
        Ipv6Addr, 
        Ipv4Addr, 
        IpAddr, 
    }, 

    net::{
        SocketAddrV6,
        SocketAddrV4, 
        SocketAddr, 
    }, 

    time::{
        Duration, 
        Instant,
    },

};

use socket2::{
    Protocol, 
    Domain, 
    Socket, 
    Type,
};

use trust_dns_resolver::{

    Resolver, 

    config::{
        ResolverConfig, 
        ResolverOpts,
    },

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
    resolver_cache: HashMap<IpAddr, Option<String>>,
    resolver: Resolver,
    source_socket_address: SocketAddr,
    target_socket_address: SocketAddr,
    socket: Socket,
    buffer_receive: Vec<u8>,
    buffer_send: Vec<u8>,
    sequence_number: u16,
}

impl Ekko {
    fn resolve_domain(&mut self, target: IpAddr) -> Result<Option<String>, EkkoError> {
        let domain = if self.resolver_cache.contains_key(&(target)) {
            self.resolver_cache.get(&target).ok_or({
                EkkoError::ResolverDomainCacheLookup(target.to_string())
            })?.clone()
        } 
        
        else {
            if let Ok(entries) = self.resolver.reverse_lookup(target) {
                if let Some(result) = entries.iter().next() {
                    self.resolver_cache.insert(target, Some(result.to_string()));
                    Some(result.to_string())
                } else {
                    self.resolver_cache.insert(target, None);
                    None
                }
            } else {
                self.resolver_cache.insert(target, None);
                None
            }
        };

        Ok(domain)
    }

    /// Send an echo request with a default timeout of 250 milliseconds ..
    pub fn send(&mut self, hops: u32) -> Result<EkkoResponse, EkkoError> {
        self.send_with_timeout(hops, Some(Duration::from_millis(256)))
    }

    /// Send an echo request with or without a timeout ..
    pub fn send_with_timeout(&mut self, hops: u32, timeout: Option<Duration>) -> Result<EkkoResponse, EkkoError> {
        self.socket.set_read_timeout(timeout).map_err(|e| {
            EkkoError::SocketSetReadTimeout(e.to_string())
        })?;

        self.socket.set_ttl(hops).map_err(|e| {
            EkkoError::SocketSetMaxHops(e.to_string())
        })?;

        let timepoint = Instant::now();
        match (self.source_socket_address, self.target_socket_address) {
            (SocketAddr::V6(source), SocketAddr::V6(target)) => {
                self.buffer_send.resize(64, 0);
                let request = EchoRequest::new_ipv6(self.buffer_send.as_mut_slice(), rand::random(), self.sequence_number, &source.ip().segments(), &target.ip().segments())?;
                self.socket.send_to(request.as_slice(), &(target.into())).map_err(|e| {
                    EkkoError::SocketSend(e.to_string())
                })?;
            },
            (SocketAddr::V4(_), SocketAddr::V4(target)) => {
                self.buffer_send.resize(64, 0);
                let request = EchoRequest::new_ipv4(self.buffer_send.as_mut_slice(), rand::random(), self.sequence_number)?;
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

        self.sequence_number += 1;
        self.buffer_receive.resize(512, 0);
        if let Ok((_, responder)) = self.socket.recv_from(self.buffer_receive.as_mut_slice())  {
            let responding_address = match self.source_socket_address {
                SocketAddr::V6(_) => IpAddr::V6(responder.as_inet6()
                    .ok_or(EkkoError::SocketReceiveNoIpv6)?.ip().clone()),
                SocketAddr::V4(_) => IpAddr::V4(responder.as_inet()
                    .ok_or(EkkoError::SocketReceiveNoIpv4)?.ip().clone()),
            };

            let elapsed = timepoint.elapsed();
            let response = match responding_address {
                IpAddr::V4(_) => EchoResponse::from_slice(&self.buffer_receive[20..]),
                IpAddr::V6(_) => EchoResponse::from_slice(&self.buffer_receive[..]),
            };

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

                        Ok(EkkoResponse::UnreachableResponse((EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }, unreachable_code)))
                    },
                    3 => {
                        Ok(EkkoResponse::ExceededResponse(EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }))
                    },
                    129 => {
                        Ok(EkkoResponse::DestinationResponse(EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }))
                    },
                    _ => {
                        let unexpected = (response.get_type()?, response.get_code()?);

                        Ok(EkkoResponse::UnexpectedResponse((EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
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

                        Ok(EkkoResponse::UnreachableResponse((EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }, unreachable_code)))
                    },
                    11 => {
                        Ok(EkkoResponse::ExceededResponse(EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }))
                    },
                    0 => {
                        Ok(EkkoResponse::DestinationResponse(EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }))
                    },
                    _ => {
                        let unexpected = (response.get_type()?, response.get_code()?);

                        Ok(EkkoResponse::UnexpectedResponse((EkkoData { 
                            address: Some(responding_address), 
                            domain: self.resolve_domain(responding_address.clone())?, 
                            hops: hops,
                            timepoint: timepoint, 
                            elapsed: elapsed,
                        }, unexpected)))
                    },
                },
            }
        } else {
            Ok(EkkoResponse::LackingResponse(EkkoData { 
                address: None, 
                domain: None, 
                hops: hops,
                timepoint: timepoint, 
                elapsed: timepoint.elapsed(),
            }))
        }
    }

    pub fn with_target(target: &str) -> Result<Ekko, EkkoError>  {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).map_err(|e| {
            EkkoError::ResolverCreate(e.to_string())
        })?;

        let target_address = {
            if let Ok(target_address) = target.parse() {
                target_address
            } else {
                resolver.lookup_ip(target).map_err(|_| {
                    EkkoError::ResolverIpLookup(target.to_string())
                })?.iter().last().ok_or({
                    EkkoError::ResolverIpLookup(target.to_string())
                })?
            }
        };

        match target_address {
            IpAddr::V4(target_address) => {
                let source_address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
                let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).map_err(|e| {
                    EkkoError::SocketCreateIcmpv4(e.to_string())
                })?;

                socket.set_recv_buffer_size(512).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|_| {
                    EkkoError::SocketBindIpv4(source_address.to_string())
                })?;

                
                Ok(Ekko {
                    resolver_cache: HashMap::new(),
                    resolver: resolver,
                    source_socket_address: SocketAddr::V4(source_address),
                    target_socket_address: SocketAddr::V4(SocketAddrV4::new(target_address, 0)),
                    socket: socket,
                    buffer_receive: Vec::with_capacity(512),
                    buffer_send: Vec::with_capacity(64),
                    sequence_number: 0,
                })
            },
            IpAddr::V6(target_address) => {
                let source_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0, 0, 0);
                let socket = Socket::new(Domain::ipv6(), Type::raw(), Some(Protocol::icmpv6())).map_err(|e| {
                    EkkoError::SocketCreateIcmpv6(e.to_string())
                })?;

                socket.set_recv_buffer_size(512).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|_| {
                    EkkoError::SocketBindIpv6(source_address.to_string())
                })?;

                
                Ok(Ekko {
                    resolver_cache: HashMap::new(),
                    resolver: resolver,
                    source_socket_address: SocketAddr::V6(source_address),
                    target_socket_address: SocketAddr::V6(SocketAddrV6::new(target_address, 0, 0, 0)),
                    socket: socket,
                    buffer_receive: Vec::with_capacity(512),
                    buffer_send: Vec::with_capacity(64),
                    sequence_number: 0,
                })
            },
        }
    }
}
