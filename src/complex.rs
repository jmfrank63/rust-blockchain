// Not functional yet

use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, time::Duration};

use libp2p::{core::{upgrade, transport::OrTransport, muxing::StreamMuxerBox}, identity, noise, tcp, yamux, PeerId, Transport, futures::{future::Either, channel::mpsc}, gossipsub, mdns, swarm::SwarmBuilder};
use libp2p_quic as quic;
use serde::{Serialize, Deserialize};

use crate::{simple::simple_sync, network::BlockChainBehaviour, blockchain::BlockChain};

fn more_complex() {
// Create a random PeerId
    let id_keys = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {local_peer_id}");

    let (response_sender, mut response_rcv) = mpsc::unbounded();
    let (init_sender, mut init_rcv) = mpsc::unbounded();

    // Set up an encrypted DNS-enabled TCP Transport over the yamux protocol.
    let tcp_transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&id_keys).expect("signing libp2p-noise static keypair"))
        .multiplex(yamux::Config::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let quic_transport = quic::async_std::Transport::new(quic::Config::new(&id_keys));
    let transport = OrTransport::new(quic_transport, tcp_transport)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        })
        .boxed();

        // To content-address message, we can take the hash of message and use it as an ID.
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    // Set a custom gossipsub configuration
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .expect("Valid config");

    // build a gossipsub network behaviour
    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(id_keys),
        gossipsub_config,
    )
    .expect("Correct configuration");
    // Create a Gossipsub topic
    let topic = gossipsub::IdentTopic::new("test-net");
    // subscribes to our topic
    gossipsub.subscribe(&topic)?;

    let mut god_chain = BlockChain::new();
    let zeus = Gods {
        name: "Zeus".to_string(),
        from: "Greece".to_string(),
    };
    let zeus_string = serde_json::to_string(&zeus).unwrap();
    god_chain.genesis(zeus_string);

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id)?;
        let behaviour = BlockChainBehaviour::<String> {
            mdns,
            gossipsub,
            response_sender,
            init_sender,
            block_chain: god_chain,};
        SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build()
    };
}
