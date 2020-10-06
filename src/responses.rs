use std::{

    time::{
        Duration, 
        Instant,
    },

    net::{IpAddr}, 

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
pub struct EkkoData{
    pub timepoint: Instant, 
    pub elapsed: Duration,
    pub address: Option<IpAddr>, 
    pub domain: Option<String>, 
    pub hops: u32,
}

#[derive(Clone, Debug)]
pub enum EkkoResponse {
    DestinationResponse(EkkoData),
    ExceededResponse(EkkoData),
    UnreachableResponse((EkkoData, Unreachable)),
    UnexpectedResponse((EkkoData, (u8, u8))),
    LackingResponse(EkkoData),
}
