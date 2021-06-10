use std::{

    cmp::{Ordering}, 

    net::{
        
        Ipv4Addr,
        IpAddr, 
    },
    
    time::{
        
        Duration, 
        Instant,
    }};

use crate::{

    packets::{EkkoPacket},
    error::{EkkoError}, 
};

#[derive(Clone, Debug, PartialEq)]
pub enum UnreachableCodeV4 {
    CommunicationAdministrativelyProhibited,
    NetworkAdministrativelyProhibited,
    HostAdministrativelyProhibited,
    DestinationProtocolUnreachable,
    DestinationNetworkUnreachable,
    DestinationHostUnreachable,
    DestinationPortUnreachable,
    DestinationNetworkUnknown,
    HostPrecedenceViolation,
    DestinationHostUnknown,
    /// Contains next hops max transmission unit.
    FragmentationRequired(u16),
    SourceHostIsolated,
    NetworkUnreachable,
    SourceRouteFailed,
    PrecedenceCutoff,
    HostUnreachable,
    /// Contains unexpected code.
    Unexpected(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnreachableCodeV6 {
    CommunicationWithDestinationAdministrativelyProhibited,
    SourceAddressFailedIngressEgressPolicy,
    ErrorInSourceRoutingHeader,
    BeyondScopeOfSourceAddress,
    RejectRouteToDestination,
    NoRouteToDestination,
    AddressUnreachable,
    PortUnreachable,
    /// Contains unexpected code.
    Unexpected(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Unreachable {
    V4(UnreachableCodeV4),
    V6(UnreachableCodeV6),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Redirect {
    RedirectDatagramsForTypeServiceNetwork(Ipv4Addr),
    RedirectDatagramsForTypeServiceHost(Ipv4Addr),
    RedirectDatagramsForNetwork(Ipv4Addr),
    RedirectDatagramsForHost(Ipv4Addr),
    /// Contains unexpected code.
    Unexpected(u8),
}

#[derive(Clone, Debug, Eq)]
pub struct EkkoData {
    /// Timepoint for send.
    pub timepoint: Instant, 
    /// Elapsed time since send.
    pub elapsed: Duration,

    /// Responders address.
    pub address: Option<IpAddr>,

    /// Echo requests identifier.
    pub identifier: u16,
    /// Echo requests sequence.
    pub sequence: u16,
    /// Number of hops.
    pub hops: u32,
}

impl PartialEq for EkkoData {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&(other.address)) &&
        self.hops.eq(&(other.hops))
    }
}

impl PartialOrd for EkkoData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EkkoData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hops.cmp(&(other.hops))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EkkoResponse {
    Unreachable((EkkoData, Unreachable)),
    PacketTooBig(EkkoData),
    SourceQuench(EkkoData),
    Destination(EkkoData),
    Unexpected((EkkoData, (u8, u8))),
    Redirect((EkkoData, Redirect)),
    Exceeded(EkkoData),
    Lacking(EkkoData),
}

impl EkkoResponse {
    pub (crate) fn new(net: (IpAddr, u32), time: (Instant, Duration), packet: EkkoPacket) -> Result<Self, EkkoError> {
        let (timepoint, elapsed) = time;
        let (address, hops) = net;

        match address {

            IpAddr::V4(_) => match packet.get_type()? {

                3 => {

                    Ok(EkkoResponse::Unreachable(({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                        
                    }, packet.get_unreachable()?)))
                }

                4 => {

                    Ok(EkkoResponse::SourceQuench({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                    }))
                }

                5 => {

                    Ok(EkkoResponse::Redirect(({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }

                    }, packet.get_redirect()?)))
                }

                11 => {

                    Ok(EkkoResponse::Exceeded({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                    }))
                }

                0 => {

                    Ok(EkkoResponse::Destination({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                    }))
                }

                _ => {

                    Ok(EkkoResponse::Unexpected(({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }

                    }, (packet.get_type()?, packet.get_code()?))))
                }
            }

            IpAddr::V6(_) => match packet.get_type()? {

                1 => {

                    Ok(EkkoResponse::Unreachable(({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }

                    }, packet.get_unreachable()?)))
                }

                2 => {

                    Ok(EkkoResponse::PacketTooBig({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                    }))
                }

                3 => {

                    Ok(EkkoResponse::Exceeded({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                    }))
                }

                129 => {

                    Ok(EkkoResponse::Destination({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }
                    }))
                }

                _ => {

                    Ok(EkkoResponse::Unexpected(({

                        EkkoData { 

                            timepoint, 
                            elapsed,
                            
                            address: Some(address),

                            identifier: packet.get_identifier()?,
                            sequence: packet.get_sequence()?,
                            hops,
                        }

                    }, (packet.get_type()?, packet.get_code()?))))
                }
            }
        }
    }
}
