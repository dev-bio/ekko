use std::{

    cmp::{Ordering}, 
    net::{IpAddr}, 
    
    time::{
        Duration, 
        Instant,
    },
};

use crate::error::{EkkoError};

#[derive(Clone, Debug)]
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
    FragmentationRequired,
    SourceHostIsolated,
    NetworkUnreachable,
    SourceRouteFailed,
    PrecedenceCutoff,
    HostUnreachable,
    Unexpected(u8),
}

#[derive(Clone, Debug)]
pub enum UnreachableCodeV6 {
    CommunicationWithDestinationAdministrativelyProhibited,
    SourceAddressFailedIngressEgressPolicy,
    ErrorInSourceRoutingHeader,
    BeyondScopeOfSourceAddress,
    RejectRouteToDestination,
    NoRouteToDestination,
    AddressUnreachable,
    PortUnreachable,
    Unexpected(u8),
}

#[derive(Clone, Debug)]
pub enum Unreachable {
    V4(UnreachableCodeV4),
    V6(UnreachableCodeV6),
}

#[derive(Clone, Debug)]
pub enum ParameterProblemV4 {
    Pointer,
    Unexpected(u8),
}

#[derive(Clone, Debug)]
pub enum ParameterProblemV6 {
    UnrecognizedNextHeaderType,
    ErroneousHeaderField,
    UnrecognizedOption,
    Unexpected(u8),
}

#[derive(Clone, Debug)]
pub enum ParameterProblem {
    V4(ParameterProblemV4),
    V6(ParameterProblemV6),
}

#[derive(Clone, Debug)]
pub enum Redirect {
    RedirectDatagramsForTypeServiceNetwork,
    RedirectDatagramsForTypeServiceHost,
    RedirectDatagramsForNetwork,
    RedirectDatagramsForHost,
    Unexpected(u8),
}

#[derive(Clone, Debug, Eq)]
pub struct EkkoData{
    pub timepoint: Instant, 
    pub elapsed: Duration,
    pub address: Option<IpAddr>,
    pub hops: u32,
}

impl Ord for EkkoData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hops.cmp(&(other.hops))
    }
}

impl PartialOrd for EkkoData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for EkkoData {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&(other.address)) &&
        self.hops.eq(&(other.hops))
    }
}

#[derive(Clone, Debug)]
pub enum EkkoResponse {
    ParameterProblem((EkkoData, ParameterProblem)),
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
    pub fn new(address: IpAddr, hops: u32, response_type: u8, response_code: u8, timepoint: Instant, elapsed: Duration) -> Result<Self, EkkoError> {
        match address {
            IpAddr::V4(_) => match response_type {

                3 => {

                    let unreachable_code = Unreachable::V4(match response_code {
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
                        _ => UnreachableCodeV4::Unexpected(response_code),
                    });

                    Ok(EkkoResponse::Unreachable((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, unreachable_code)))
                }

                4 => {

                    Ok(EkkoResponse::SourceQuench(EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }))
                }

                5 => {

                    let redirect_code = match response_code {
                        0 => Redirect::RedirectDatagramsForNetwork,
                        1 => Redirect::RedirectDatagramsForHost,
                        2 => Redirect::RedirectDatagramsForTypeServiceNetwork,
                        3 => Redirect::RedirectDatagramsForTypeServiceHost,
                        _ => Redirect::Unexpected(response_code),
                    };

                    Ok(EkkoResponse::Redirect((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, redirect_code)))
                }

                11 => {

                    Ok(EkkoResponse::Exceeded(EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }))
                }

                12 => {
                    
                    let parameter_problem_code = ParameterProblem::V4(match response_code {
                        0 => ParameterProblemV4::Pointer,
                        _ => ParameterProblemV4::Unexpected(response_code),
                    });

                    Ok(EkkoResponse::ParameterProblem((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, parameter_problem_code)))
                }

                0 => {

                    Ok(EkkoResponse::Destination(EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }))
                }

                _ => {

                    let unexpected = (response_type, response_code);

                    Ok(EkkoResponse::Unexpected((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, unexpected)))
                }
            }

            IpAddr::V6(_) => match response_type {

                1 => {

                    let unreachable_code = Unreachable::V6(match response_code {
                        0 => UnreachableCodeV6::NoRouteToDestination,
                        1 => UnreachableCodeV6::CommunicationWithDestinationAdministrativelyProhibited,
                        2 => UnreachableCodeV6::BeyondScopeOfSourceAddress,
                        3 => UnreachableCodeV6::AddressUnreachable,
                        4 => UnreachableCodeV6::PortUnreachable,
                        5 => UnreachableCodeV6::SourceAddressFailedIngressEgressPolicy,
                        6 => UnreachableCodeV6::RejectRouteToDestination,
                        7 => UnreachableCodeV6::ErrorInSourceRoutingHeader,
                        _ => UnreachableCodeV6::Unexpected(response_code),
                    });

                    Ok(EkkoResponse::Unreachable((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, unreachable_code)))
                }

                2 => {

                    Ok(EkkoResponse::PacketTooBig(EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }))
                }

                3 => {

                    Ok(EkkoResponse::Exceeded(EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }))
                }

                4 => {

                    let parameter_problem_code = ParameterProblem::V6(match response_code {
                        0 => ParameterProblemV6::ErroneousHeaderField,
                        1 => ParameterProblemV6::UnrecognizedNextHeaderType,
                        2 => ParameterProblemV6::UnrecognizedOption,
                        _ => ParameterProblemV6::Unexpected(response_code),
                    });

                    Ok(EkkoResponse::ParameterProblem((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, parameter_problem_code)))
                }

                129 => {

                    Ok(EkkoResponse::Destination(EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }))
                }

                _ => {

                    let unexpected = (response_type, response_code);

                    Ok(EkkoResponse::Unexpected((EkkoData { 
                        timepoint: timepoint, 
                        elapsed: elapsed,
                        address: Some(address),
                        hops: hops,
                    }, unexpected)))
                }
            }
        }
    }
}
