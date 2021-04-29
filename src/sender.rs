use std::{
    
    ops::{Range}, 
    io::{Cursor}, 

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

    socket: Socket,
}

impl Ekko {

    /// Build a sender with given target address ..
    pub fn with_target<T: Into<SocketAddr>>(target: T) -> Result<Ekko, EkkoError> {
        let target: SocketAddr = target.into();
        let result = match target.ip() {

            IpAddr::V4(_) => {

                let source_address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
                let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).map_err(|e| {
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

                    socket: socket,
                }
            }

            IpAddr::V6(_) => {

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
  
                Ekko {

                    source_socket_address: SocketAddr::V6(source_address),
                    target_socket_address: target,

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
    pub fn send_with_timeout(&self, hops: u32, timeout: Duration) -> Result<EkkoResponse, EkkoError> {
        let identifier = rand::random();
        let mut sequence = 0;

        let timepoint = Instant::now();

        self.inner_send(hops, sequence, identifier)?;
        sequence = sequence.wrapping_add(1);

        let result = loop {

            let mut buffer: [u8; 512] = [0; 512];

            let identifier = identifier;
            if let Some((address, response)) = self.inner_recv(&mut buffer[..])? {
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

            break EkkoResponse::Lacking({

                EkkoData { 

                    address: None,
                    hops: hops,
                    
                    timepoint: timepoint, 
                    elapsed: timepoint.elapsed(),
                }
            })
        };

        Ok(result)
    }

    /// Send echo requests for all hops in range at the same time with a default timeout of 1000 milliseconds. Note that 
    /// the target may end up being flooded with echo requests if the range is way above the needed hops to reach it!
    pub fn send_range(&self, hops: Range<u32>) -> Result<Vec<EkkoResponse>, EkkoError> {
        self.send_range_with_timeout(hops, Duration::from_millis(1000))
    }

    /// Send echo requests for all hops in range at the same time with specified timeout. Note that the target may end up 
    /// being flooded with echo requests if the range is way above the needed hops to reach it!
    pub fn send_range_with_timeout(&self, hops: Range<u32>, timeout: Duration) -> Result<Vec<EkkoResponse>, EkkoError> {
        let mut echo_responses = Vec::with_capacity(hops.len());
        let mut echo_requests = Vec::with_capacity(hops.len());
        let mut echo_route = Vec::with_capacity(hops.len());
        
        let identifier = rand::random();       
        let mut sequence = 0;

        let timepoint = Instant::now();

        for hop in hops {

            self.inner_send(hop, sequence, identifier)?;
            sequence = sequence.wrapping_add(1);

            echo_requests.push({
                (timepoint.clone(), sequence.clone(), hop.clone())
            });
        }

        loop {

            let mut buffer: [u8; 512] = [0; 512];
            if let Some((address, response)) = self.inner_recv(&mut buffer[..])? {
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

                        EkkoResponse::Lacking({

                            EkkoData { 

                                address: None,
                                hops: request_hops.clone(),

                                timepoint: request_timepoint.clone(), 
                                elapsed: request_timepoint.elapsed(),
                            }
                        })
                    });
                }
            }

            break Ok(echo_route)
        }
    }

    fn inner_send(&self, hops: u32, seq: u16, idf: u16) -> Result<(), EkkoError> {
        match (self.source_socket_address, self.target_socket_address) {

            (SocketAddr::V6(_), SocketAddr::V6(_)) => {
                self.socket.set_unicast_hops_v6(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv6(e.to_string())
                })?;
            }

            (SocketAddr::V4(_), SocketAddr::V4(_)) => {
                self.socket.set_ttl(hops).map_err(|e| {
                    EkkoError::SocketSetMaxHopsIpv4(e.to_string())
                })?;
            }

            (src, dst) => {
                return Err(EkkoError::SocketIpMismatch { 
                    src: src.to_string(), 
                    dst: dst.to_string() 
                })
            }
        };

        let mut buffer: [u8; 512] = [0; 512];
        let request = EchoRequest::new(&mut buffer[..], idf, seq, self.source_socket_address, self.target_socket_address)?;
        self.socket.send_to(request.as_slice(), &(self.target_socket_address.into())).map_err(|e| {
            EkkoError::SocketSend(e.to_string())
        })?;

        Ok(())
    }

    fn inner_recv<'a>(&self, buf: &'a mut [u8]) -> Result<Option<(IpAddr, EchoResponse<'a>)>, EkkoError> {
        let result = self.socket.recv_from(unsafe { 
            std::mem::transmute(&mut buf[..]) 
        });

        if let Ok((_, responder)) = result {
            let responding_address = match self.source_socket_address {

                SocketAddr::V4(_) => IpAddr::V4(responder.as_socket_ipv4()
                    .ok_or(EkkoError::SocketReceiveNoIpv4)?.ip().clone()),
                    
                SocketAddr::V6(_) => IpAddr::V6(responder.as_socket_ipv6()
                    .ok_or(EkkoError::SocketReceiveNoIpv6)?.ip().clone()),
            };

            let mut cursor = Cursor::new(buf.as_ref());
            let header_octets = ((cursor.read_u8().map_err(|e| {
                EkkoError::ResponseReadField("internet protocol header size", e.to_string())
            })? & 0x0F) * 4) as usize;

            match responding_address {

                IpAddr::V4(_) => Ok(Some((responding_address, {
                    EchoResponse::V4(&buf[header_octets..])
                }))),

                IpAddr::V6(_) => Ok(Some((responding_address, {
                    EchoResponse::V6(&buf[..])
                }))),
            }
        }
        
        else {
            
            Ok(None)
        }
    }
}
