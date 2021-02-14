use std::{

    cmp::{Ordering}, 
    net::{IpAddr}, 
    
    time::{
        Duration, 
        Instant,
    },
};

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
