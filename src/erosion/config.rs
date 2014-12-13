use std::time::duration::Duration;
use std::io::net::ip::{
    Ipv4Addr,
    SocketAddr,
};

#[deriving(Clone)]
pub struct Config {
    pub name: String,
    pub bind_addr: SocketAddr,

    /// The timeout for establishing a TCP connection with a remote node for
    /// a full state sync.
    tcp_timeout: Duration,

    pub indirect_checks: uint,

    /// The multiplier for the number of retransmissions that are attempted for
    /// messages broadcasted over gossip.
    ///
    /// The actual count of retransmissions is calculated using the formula:
    ///
    ///   retransmits = retransmit_mult * log(N+1)
    ///
    /// This allows the retransmits to scale properly with cluster size. The
    /// higher the multiplier, the more likely a failed broadcast is to converge
    /// at the expense of increased bandwidth.
    retransmit_mult: int,

    /// The multiplier for determining the time an inaccessible node is
    /// considered suspect before declaring it dead.
    ///
    /// The actual timeout is calculated using the formula:
    ///
    ///   suspicion_timeout = suspicion_mult * log(N+1) * probe_interval
    ///
    /// This allows the timeout to scale properly with expected propagation
    /// delay with a larger cluster size. The higher the multiplier, the longer
    /// an inaccessible node is considered part of the cluster before declaring
    /// it dead, giving that suspect node more time to refute if it is indeed
    /// still alive.
    suspicion_mult: int,

    /// The interval between complete state syncs. Complete state syncs are
    /// done with a single node over TCP and are quite expensive relative to
    /// standard gossiped messages. Setting this to zero will disable state
    /// push/pull syncs completely.
    ///
    /// Setting this interval lower (more frequent) will increase convergence
    /// speeds across larger clusters at the expense of increased bandwidth
    /// usage.
    push_pull_interval: Duration,

    /// The interval between random node probes. Setting this lower (more
    /// frequent) will cause the memberlist cluster to detect failed nodes
    /// more quickly at the expense of increased bandwidth usage.
    pub probe_interval: Duration,

    /// The timeout to wait for an ack from a probed node before assuming it
    /// is unhealthy. This should be set to 99-percentile of RTT (round-trip
    /// time) on your network.
    pub probe_timeout: Duration,

    /// The interval between sending messages that need
    /// to be gossiped that haven't been able to piggyback on probing messages.
    /// If this is set to zero, non-piggyback gossip is disabled. By lowering
    /// this value (more frequent) gossip messages are propagated across
    /// the cluster more quickly at the expense of increased bandwidth.
    pub gossip_interval: Duration,

    /// The number of random nodes to send gossip messages to
    /// per GossipInterval. Increasing this number causes the gossip messages
    /// to propagate across the cluster more quickly at the expense of
    /// increased bandwidth.
    pub gossip_nodes: uint,

    /// Used to control message compression. This can be used to reduce
    /// bandwidth usage at the cost of slightly more CPU utilization.
    enable_compression: bool,
}

/// Returns a sane set of configurations.
///
/// It sets very conservative values that are sane for most LAN environments.
/// The default configuration errs on the side on the side of caution,
/// choosing values that are optimized for higher convergence at the cost of
/// higher bandwidth usage. Regardless, these values are a good starting
/// point when getting started with memberlist.
pub fn lan(name: String) -> Config {
    Config {
        name: name,
        bind_addr: SocketAddr {
            ip: Ipv4Addr(0, 0, 0, 0),
            port: 7201,
        },
        tcp_timeout: Duration::seconds(10),
        indirect_checks: 3,
        retransmit_mult: 4,
        suspicion_mult: 5,
        push_pull_interval: Duration::seconds(30),
        probe_interval: Duration::seconds(1),
        probe_timeout: Duration::milliseconds(500),
        gossip_interval: Duration::milliseconds(200),
        gossip_nodes: 3,
        enable_compression: true,
    }
}

/// Like `lan`, however it returns a configuration that is optimized for
/// most WAN environments. The default configuration is still very
/// conservative and errs on the side of caution.
pub fn wan(name: String) -> Config {
    let mut config = lan(name);
    config.tcp_timeout = Duration::seconds(30);
    config.suspicion_mult = 6;
    config.push_pull_interval = Duration::seconds(60);
    config.probe_interval = Duration::seconds(5);
    config.probe_timeout = Duration::seconds(3);
    config.gossip_interval = Duration::milliseconds(500);
    config.gossip_nodes = 4;
    config
}

/// Like `lan`, however it returns a configuration that is optimized for a
/// local loopback environments. The default configuration is still very
/// conservative and errs on the side of caution.
pub fn local(name: String) -> Config {
    let mut config = lan(name);
    config.tcp_timeout = Duration::seconds(1);
    config.indirect_checks = 1;
    config.retransmit_mult = 2;
    config.suspicion_mult = 3;
    config.push_pull_interval = Duration::seconds(15);
    config.probe_timeout = Duration::milliseconds(200);
    config.gossip_interval = Duration::milliseconds(100);
    config
}
