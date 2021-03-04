use std::{

    io::{Cursor}, 

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
    
    ops::{Range}, 
    
    time::{
        Duration, 
        Instant,
    }
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
    },
};

pub struct Ekko {

    source_socket_address: SocketAddr,
    target_socket_address: SocketAddr,

    identifier: u16,
    sequence: u16,

    buffer_receive: Vec<u8>,
    buffer_send: Vec<u8>,
    socket: Socket,
}

impl Ekko {

    /// Build a sender with given target address ..
    pub fn with_target(target: &str) -> Result<Ekko, EkkoError> {
        let target = target.to_socket_addrs().or_else(|_| {
            format!("[{}]:0", target).to_socket_addrs().or_else(|_| {
                format!("{}:0", target).to_socket_addrs().map_err(|e| {
                    EkkoError::BadTarget(target.to_string(), e.to_string())
                })
            }).map_err(|e| {
                EkkoError::BadTarget(target.to_string(), e.to_string())
            })
        }).and_then(|results| results.into_iter().next().ok_or({
            EkkoError::UnresolvedTarget(target.to_string())
        })).and_then(|result| Ok(result))?;

        let result = match target.ip() {

            IpAddr::V4(_) => {

                let source_address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
                let socket = Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())).map_err(|e| {
                    EkkoError::SocketCreateIcmpv4(e.to_string())
                })?;

                socket.set_nonblocking(true).map_err(|e| {
                    EkkoError::SocketSetNonBlocking(true, e.to_string())
                })?;

                socket.set_recv_buffer_size(512).map_err(|e| {
                    EkkoError::SocketSetReceiveBufferSize(e.to_string())
                })?;

                socket.bind(&(source_address.into())).map_err(|e| {
                    EkkoError::SocketBindIpv4(source_address.to_string(), e.to_string())
                })?;
                
                Ekko {

                    source_socket_address: SocketAddr::V4(source_address),
                    target_socket_address: target,

                    identifier: rand::random(),
                    sequence: 0,

                    buffer_receive: Vec::with_capacity(512),
                    buffer_send: Vec::with_capacity(64),
                    socket: socket,
                }
            }

            IpAddr::V6(_) => {

                let source_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0, 0, 0);
                let socket = Socket::new(Domain::ipv6(), Type::raw(), Some(Protocol::icmpv6())).map_err(|e| {
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
  
                Ekko {

                    source_socket_address: SocketAddr::V6(source_address),
                    target_socket_address: target,

                    identifier: rand::random(),
                    sequence: 0,

                    buffer_receive: Vec::with_capacity(512),
                    buffer_send: Vec::with_capacity(64),
                    socket: socket,
                }
            }
        };

        Ok(result)
    }

    /// Send an echo request with a default timeout of 1000 milliseconds ..
    pub fn send(&mut self, hops: u32) -> Result<EkkoResponse, EkkoError> {
        self.send_with_timeout(hops, Duration::from_millis(1000))
    }

    /// Send an echo request with or with a specified timeout ..
    pub fn send_with_timeout(&mut self, hops: u32, timeout: Duration) -> Result<EkkoResponse, EkkoError> {
        let identifier = self.identifier;
        let sequence = self.sequence;

        let timepoint = Instant::now();

        self.inner_send(hops, sequence, identifier)?;
        self.sequence = sequence.wrapping_add(1);

        let result = loop {

            let identifier = self.identifier;
            if let Some((address, response)) = self.inner_recv()? {
                if identifier == response.get_identifier()? {
                    if sequence == response.get_sequence()? {
                        
                        let detail = (response.get_type()?, response.get_code()?);
                        let time = (timepoint.clone(), timepoint.elapsed());
                        let net = (address.clone(), hops.clone());

                        break EkkoResponse::new(net, detail, time)?;
                    }
                }
            }

            if timepoint.elapsed() < timeout {
                continue
            }

            break EkkoResponse::Lacking(EkkoData { 
                timepoint: timepoint, 
                elapsed: timepoint.elapsed(),
                address: None,
                hops: hops,
            })
        };

        Ok(result)
    }

    /// Trace route with a default timeout of 1000 milliseconds ..
    pub fn trace(&mut self, hops: Range<u32>) -> Result<Vec<EkkoResponse>, EkkoError> {
        self.trace_with_timeout(hops, Duration::from_millis(1000))
    }

    /// Trace route with specified timeout ..
    pub fn trace_with_timeout(&mut self, hops: Range<u32>, timeout: Duration) -> Result<Vec<EkkoResponse>, EkkoError> {
        let mut echo_responses = Vec::new();
        let mut echo_requests = Vec::new();
        let mut echo_route = Vec::new();
        
        let timepoint = Instant::now();

        for hop in hops {
            let identifier = self.identifier;       
            let sequence = self.sequence;

            self.inner_send(hop, sequence, identifier)?;
            self.sequence = sequence.wrapping_add(1);

            echo_requests.push({
                (timepoint.clone(), sequence.clone(), hop.clone())
            });
        }

        loop {

            let identifier = self.identifier;
            if let Some((address, response)) = self.inner_recv()? {
                for (request_timepoint, request_sequence, _) 
                    in &(echo_requests) {

                    match (request_sequence.clone(), identifier.clone()) {

                        x if x == (response.get_sequence()?, response.get_identifier()?) => echo_responses.push({
                            (address.clone(), request_timepoint.clone(), request_timepoint.elapsed(), response.get_type()?, response.get_code()?, response.get_sequence()?)
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
            
            for (request_timepoint, request_sequence, request_hops) 
                in echo_requests {

                let previous_route_length = echo_route.len();
                for (response_address, response_timepoint, response_elapsed, response_type, response_code, response_sequence) 
                    in &(echo_responses) {

                    match request_sequence {

                        ref x if x == response_sequence => echo_route.push({
                            let detail = (response_type.clone(), response_code.clone());
                            let time = (response_timepoint.clone(), response_elapsed.clone());
                            let net = (response_address.clone(), request_hops.clone());
    
                            EkkoResponse::new(net, detail, time)?
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
                        EkkoResponse::Lacking(EkkoData { 
                            address: None,
                            hops: request_hops.clone(),
                            timepoint: request_timepoint.clone(), 
                            elapsed: request_timepoint.elapsed(),
                        })
                    });
                }
            }

            break Ok(echo_route)
        }
    }

    fn inner_send(&mut self, hops: u32, req_sequence: u16, req_identifier: u16) -> Result<(), EkkoError> {
        match (self.source_socket_address, self.target_socket_address) {

            (SocketAddr::V6(source), SocketAddr::V6(target)) => {
                self.socket.set_unicast_hops_v6(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv6(e.to_string())
                })?;

                self.buffer_send.resize(64, 0);
                let request = EchoRequest::new_ipv6(self.buffer_send.as_mut_slice(), req_identifier, req_sequence, &(source.ip().segments()), &(target.ip().segments()))?;
                self.socket.send_to(request.as_slice(), &(target.into())).map_err(|e| {
                    EkkoError::SocketSend(e.to_string())
                })?;
            }

            (SocketAddr::V4(_), SocketAddr::V4(target)) => {
                self.socket.set_ttl(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv4(e.to_string())
                })?;

                self.buffer_send.resize(64, 0);
                let request = EchoRequest::new_ipv4(self.buffer_send.as_mut_slice(), req_identifier, req_sequence)?;
                self.socket.send_to(request.as_slice(), &(target.into())).map_err(|e| {
                    EkkoError::SocketSend(e.to_string())
                })?;
            }

            (SocketAddr::V4(source), SocketAddr::V6(target)) => {
                return Err(EkkoError::SocketIpMismatch { 
                    src: source.to_string(), 
                    tgt: target.to_string() 
                })
            }

            (SocketAddr::V6(source), SocketAddr::V4(target)) => {
                return Err(EkkoError::SocketIpMismatch { 
                    src: source.to_string(), 
                    tgt: target.to_string() 
                })
            }
        };

        Ok(())
    }

    fn inner_recv(&mut self) -> Result<Option<(IpAddr, EchoResponse)>, EkkoError> {
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

            match responding_address {

                IpAddr::V4(_) => Ok(Some((responding_address, EchoResponse::V4(&self.buffer_receive[header_octets..])))),
                IpAddr::V6(_) => Ok(Some((responding_address, EchoResponse::V6(&self.buffer_receive[..])))),
            }
        }
        
        else {
            
            Ok(None)
        }
    }
}
