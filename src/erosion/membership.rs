use std::collections::HashMap;
use std::io::timer::{
    sleep,
    Timer,
};
use std::io::net::ip::SocketAddr;
use std::rand::{
    task_rng,
    Rng,
};
use std::sync::{
    Arc,
    RWLock,
    Mutex,
};
use std::time::Duration;

use member::{
    Member,
    MemberState,
};

use gossip;
use gossip::Gossip;

use message::Message;

use config::Config;


pub struct Membership {
    meta: Arc<MembershipMeta>,

    /// Local incarnation number
    inc: u32,
}

impl Membership {
    /// Create the network listeners
    pub fn bind(config: Config) -> Result<Membership, String> {
        let gossip = Gossip::new(config.bind_addr);
        if let Err(e) = gossip {
            return Err(e);
        }

        let members = Vec::new();

        Ok(Membership {
            meta: Arc::new(MembershipMeta {
                config: config,
                members: RWLock::new(members),
                gossip: Mutex::new(gossip.unwrap()),

                ack_senders: Arc::new(Mutex::new(HashMap::new())),

                seq: Mutex::new(0),
                probe_index: Mutex::new(0),
            }),

            inc: 0,
        })
    }

    pub fn start(&mut self) {
        self.start_gossip_listening();
        self.start_probing();
    }

    /// Join a existing cluster
    pub fn join(&mut self, name: String, addr: SocketAddr) {
        let member = Member {
            name: name,
            addr: addr,
            state: MemberState::Alive,
            inc: 0,
        };

        {
            let mut members = self.meta.members.write();
            members.push(member);
        }

        self.start();
    }

    fn start_probing(&mut self) {
        if self.meta.config.probe_interval.is_zero() {
            return;
        }

        let meta = self.meta.clone();
        let duration = self.meta.config.probe_interval;

        spawn(proc() {
            loop {
                sleep(duration);
                meta.probe();
            }
        });
    }

    fn start_gossip_listening(&mut self) {
        let meta = self.meta.clone();

        spawn(proc() {
            loop {
                if let Some((msg, from)) = meta.recv_msg() {
                    meta.handle_message(msg, from);
                }
            }
        })
    }
}

struct MembershipMeta {
    config: Config,

    members: RWLock<Vec<Member>>,

    gossip: Mutex<Gossip>,

    ack_senders: Arc<Mutex<HashMap<u32, Sender<()>>>>,

    /// local sequence number
    seq: Mutex<u32>,

    probe_index: Mutex<uint>,
}

impl MembershipMeta {
    fn recv_msg(&self) -> Option<(Message, SocketAddr)> {
        let mut gossip = self.gossip.lock();

        gossip.udp.set_read_timeout(Some(1));

        let mut buf = [0u8, ..gossip::UDP_MAX_SIZE];
        let result = gossip.udp.recv_from(&mut buf);
        if let Err(_) = result {
            return None;
        }

        let (count, from) = result.unwrap();
        let mut buf = buf[..count];
        match Message::read(&mut buf) {
            Ok(msg) => {
                println!("Received message from {} => {}", from, msg);
                Some((msg, from))
            },
            Err(e) => {
                println!("Failed to decode message from {} => {}", from, e);
                None
            },
        }
    }

    fn handle_message(&self, msg: Message, from: SocketAddr) {
        match msg {
            Message::Ping {
                seq,
                name,
            } => {
                if name != self.config.name {
                    println!("Got ping for unexpected member `{}`", name);
                    return;
                }
                self.gossip.lock().ack(seq, from);
            },

            Message::Ack {
                seq,
            } => {
                if let Some(sender) = self.ack_senders.lock().remove(&seq) {
                    sender.send(());
                }
            },

            _ => {},
        }
    }

    /// Used to perform a single round of failure detection and gossip
    fn probe(&self) {
        let probe_index = { *self.probe_index.lock() };
        let members_len = { self.members.read().len() };

        if probe_index >= members_len {
            self.reset_members();
            (*self.probe_index.lock()) = 0;
        }

        let mut member_to_be_probed = None;
        {
            let members = self.members.read();
            let mut probe_index = self.probe_index.lock();
            while *probe_index < members.len() {
                let member = &members[*probe_index];
                (*probe_index) += 1;
                if member.name == self.config.name
                   || member.state == MemberState::Dead {
                    continue;
                }
                member_to_be_probed = Some((*member).clone())
            }
        }

        if let Some(member) = member_to_be_probed {
            self.probe_member(member);
        }
    }

    fn probe_member(&self, member: Member) {
        println!("Start probing {}", member);
        let seq = self.next_seq();
        self.gossip.lock().ping(seq, member.name, member.addr);
        if let Some(_) = self.wait_ack(seq, self.config.probe_timeout) {
            println!("Ack {} confirmed.", seq);
            return;
        }

        println!("Ack {} timeout.", seq);
    }

    fn wait_ack(&self, seq: u32, duration: Duration) -> Option<()> {
        let (tx, rx) = channel();
        self.ack_senders.lock().insert(seq, tx);
        let mut timer = Timer::new().unwrap();
        let timeout = timer.oneshot(duration);

        select!(
            () = rx.recv() => Some(()),
            () = timeout.recv() => {
                self.ack_senders.lock().remove(&seq);
                None
            }
        )
    }

    /// Used when the `probe_index` wraps around. It will reap the dead members
    /// and shuffle the member list
    fn reset_members(&self) { // TODO
        let mut rng = task_rng();
        rng.shuffle(self.members.write().as_mut_slice());
    }

    fn next_seq(&self) -> u32 {
        let mut seq = self.seq.lock();
        (*seq) += 1;
        *seq
    }
}
