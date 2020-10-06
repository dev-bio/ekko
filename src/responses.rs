use std::{

    time::{
        Duration, 
        Instant,
    },

    net::{IpAddr}, 

};

#[derive(Clone, Debug)]
pub enum UnreachableCodeV4 {
    DestinationNetworkUnreachable,
    DestinationHostUnreachable,
    DestinationProtocolUnreachable,
    DestinationPortUnreachable,
    FragmentationRequired,
    SourceRouteFailed,
    DestinationNetworkUnknown,
    DestinationHostUnknown,
    SourceHostIsolated,
    NetworkAdministrativelyProhibited,
    HostAdministrativelyProhibited,
    NetworkUnreachable,
    HostUnreachable,
    CommunicationAdministrativelyProhibited,
    HostPrecedenceViolation,
    PrecedenceCutoff,
    Unexpected(u8),
}

#[derive(Clone, Debug)]
pub enum UnreachableCodeV6 {
    NoRouteToDestination,
    CommunicationWithDestinationAdministrativelyProhibited,
    BeyondScopeOfSourceAddress,
    AddressUnreachable,
    PortUnreachable,
    SourceAddressFailedIngressEgressPolicy,
    RejectRouteToDestination,
    ErrorInSourceRoutingHeader,
    Unexpected(u8),
}

#[derive(Clone, Debug)]
pub enum Unreachable {
    V4(UnreachableCodeV4),
    V6(UnreachableCodeV6),
}

#[derive(Clone, Debug)]
pub struct EkkoData{
    pub address: Option<IpAddr>, 
    pub domain: Option<String>, 
    pub hops: u32,
    pub timepoint: Instant, 
    pub elapsed: Duration,
}

#[derive(Clone, Debug)]
pub enum EkkoResponse {
    DestinationResponse(EkkoData),
    ExceededResponse(EkkoData),
    UnreachableResponse((EkkoData, Unreachable)),
    UnexpectedResponse((EkkoData, (u8, u8))),
    LackingResponse(EkkoData),
}
