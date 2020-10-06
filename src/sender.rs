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

use anyhow::bail;
use anyhow::{
    Context, 
    Result, 
};

use trust_dns_resolver::{

    Resolver, 

    config::{
        ResolverConfig, 
        ResolverOpts,
    },

};

use super::{

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
    buffer: Vec<u8>,
    sequence_number: u16,
}

impl Ekko {
    fn resolve_domain(&mut self, target: IpAddr) -> Result<Option<String>> {
        let domain = if self.resolver_cache.contains_key(&target) {
            self.resolver_cache.get(&target).context("Getting domain from cache.")?.clone()
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

    pub fn send(&mut self, hops: u32) -> Result<EkkoResponse> {
        self.send_with_timeout(hops, Some(Duration::from_millis(256)))
    }

    pub fn send_with_timeout(&mut self, hops: u32, timeout: Option<Duration>) -> Result<EkkoResponse> {
        self.socket.set_recv_buffer_size(512)?;
        self.socket.set_read_timeout(timeout)?;
        self.socket.set_ttl(hops)?;

        let timepoint = Instant::now();
        match (self.source_socket_address, self.target_socket_address) {
            (SocketAddr::V6(source), SocketAddr::V6(target)) => {
                self.buffer.resize(512, 0);
                let request = EchoRequest::new_ipv6(self.buffer.as_mut_slice(), rand::random(), self.sequence_number, &source.ip().segments(), &target.ip().segments())?;
                self.socket.send_to(&(request.as_slice()[..64]), &(target.into()))?;
            },
            (SocketAddr::V4(_), SocketAddr::V4(target)) => {
                self.buffer.resize(512, 0);
                let request = EchoRequest::new_ipv4(self.buffer.as_mut_slice(), rand::random(), self.sequence_number)?;
                self.socket.send_to(&(request.as_slice()[..64]), &(target.into()))?;
            },
            _ => bail!("This should never happen!"),
        };

        self.buffer.resize(512, 0);
        if let Ok((_, responder)) = self.socket.recv_from(self.buffer.as_mut_slice())  {
            let responding_address = match self.source_socket_address {
                SocketAddr::V6(_) => IpAddr::V6(responder.as_inet6()
                    .context("Getting IPv6 address.")?.ip().clone()),
                SocketAddr::V4(_) => IpAddr::V4(responder.as_inet()
                    .context("Getting IPv4 address.")?.ip().clone()),
            };

            self.sequence_number += 1;

            let elapsed = timepoint.elapsed();
            let response = match responding_address {
                IpAddr::V4(_) => EchoResponse::from_slice(&self.buffer[20..]),
                IpAddr::V6(_) => EchoResponse::from_slice(&self.buffer[..]),
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

    pub fn with_target(target: &str) -> Result<Ekko>  {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;

        let target_address = {
            if let Ok(target_address) = target.parse() {
                target_address
            } else {
                resolver.lookup_ip(target)?.iter().last()
                    .context("Retrieving IP from hostname.")?
            }
        };

        match target_address {
            IpAddr::V4(target_address) => {
                let source_address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
                let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))?;

                socket.bind(&(source_address.into()))
                    .context("Binding socket to IPv4 address.")?;

                
                Ok(Ekko {
                    resolver_cache: HashMap::new(),
                    resolver: resolver,
                    source_socket_address: SocketAddr::V4(source_address),
                    target_socket_address: SocketAddr::V4(SocketAddrV4::new(target_address, 0)),
                    socket: socket,
                    buffer: Vec::with_capacity(512),
                    sequence_number: 0,
                })
            },
            IpAddr::V6(target_address) => {
                let source_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0, 0, 0);
                let socket = Socket::new(Domain::ipv6(), Type::raw(), Some(Protocol::icmpv6()))?;

                socket.bind(&(source_address.into()))
                .context("Binding socket to IPv6 address.")?;

                
                Ok(Ekko {
                    resolver_cache: HashMap::new(),
                    resolver: resolver,
                    source_socket_address: SocketAddr::V6(source_address),
                    target_socket_address: SocketAddr::V6(SocketAddrV6::new(target_address, 0, 0, 0)),
                    socket: socket,
                    buffer: Vec::with_capacity(512),
                    sequence_number: 0,
                })
            },
        }
    }
}
