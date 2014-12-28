use std::collections::HashMap;
use std::io::timer::Timer;
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
use std::thread::Thread;
use std::time::Duration;

use member::{
    Member,
    MemberState,
};

use gossip::Gossip;

use message::Message;

use config::Config;


pub struct Membership {
    started: bool,

    gossip: Gossip,

    meta: Arc<MembershipMeta>,

    message_sender: Arc<Mutex<Option<Sender<(Message, SocketAddr)>>>>,

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
            started: false,

            gossip: gossip.unwrap(),

            meta: Arc::new(MembershipMeta {
                config: config,
                members: RWLock::new(members),
                ack_senders: Mutex::new(HashMap::new()),

                seq: Mutex::new(0),
                probe_index: Mutex::new(0),
            }),

            message_sender: Arc::new(Mutex::new(None)),

            inc: 0,
        })
    }

    pub fn start(&mut self) {
        if self.started {
            return;
        }

        self.start_gossip_listening();
        self.start_probing();

        self.started = true;
    }

    /// Join a existing cluster
    pub fn join(&mut self, name: String, addr: SocketAddr) {
        if self.started {
            return;
        }

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
        let mut gossip = self.gossip.clone();
        Thread::spawn(move || {
            let mut timer = Timer::new().unwrap();
            let timeout = timer.periodic(meta.config.probe_interval);

            loop {
                meta.probe(&mut gossip);

                timeout.recv();
            }

            ()
        }).detach();
    }

    fn start_gossip_listening(&mut self) {
        let meta = self.meta.clone();
        let (message_tx, message_rx) = channel();
        let mut gossip = self.gossip.clone();
        // Handle messages
        Thread::spawn(move || {
            let (tx, rx) = channel();
            message_tx.send(tx);

            loop {
                let (msg, from) = rx.recv();
                meta.handle_message(&mut gossip, msg, from);
            }

            ()
        }).detach();

        let tx = message_rx.recv();

        let mut gossip = self.gossip.clone();
        // Receiver message from network
        Thread::spawn(move || {
            loop {
                if let Ok((msg, from)) = gossip.recv_from() {
                    tx.send((msg, from));
                }
            }

            ()
        }).detach();
    }
}

struct MembershipMeta {
    config: Config,

    members: RWLock<Vec<Member>>,

    ack_senders: Mutex<HashMap<u32, Sender<()>>>,

    /// local sequence number
    seq: Mutex<u32>,

    probe_index: Mutex<uint>,
}

impl MembershipMeta {
    fn handle_message(&self, gossip: &mut Gossip, msg: Message, from: SocketAddr) {
        match msg {
            Message::Ping {
                seq,
                name,
            } => {
                if name != self.config.name {
                    error!("Got ping for unexpected member `{}`", name);
                    return;
                }
                gossip.ack(seq, from);
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
    fn probe(&self, gossip: &mut Gossip) {
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
            self.probe_member(gossip, member);
        }
    }

    fn probe_member(&self, gossip: &mut Gossip, member: Member) {
        info!("Start probing {}", member);
        let seq = self.next_seq();
        gossip.ping(seq, member.name, member.addr);
        if let Some(_) = self.wait_ack(seq, self.config.probe_timeout) {
            info!("Ack {} confirmed.", seq);
            return;
        }

        info!("Ack {} timeout.", seq);
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
