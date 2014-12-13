use std::io::net::ip::SocketAddr;

#[deriving(Clone, Show)]
pub struct Member {
    pub name: String,
    pub addr: SocketAddr,
    pub state: MemberState,

    /// Last known incarnation number
    pub inc: u32,
}

#[deriving(PartialEq, Clone, Show)]
pub enum MemberState {
    Alive,
    Suspect,
    Dead,
}
