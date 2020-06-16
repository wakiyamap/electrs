use crate::chain::Network;
use crate::electrum::discovery::{DiscoveryManager, Service};

pub fn add_default_servers(discovery: &DiscoveryManager, network: Network) {
    match network {
        Network::Monacoin => {
            discovery.add_default_server(
                "fst2ox5p4evi5jj23kdczumzfgrrzopo7itao6fxqiowr4bufvbjkxad.onion".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "electrumx3.monacoin.nl".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "102.67.136.247".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "ri7rzlmdaf4eqbza.onion".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "133.167.67.203".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "103.125.218.246".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "ylx5ejgbmxf2m3ymdbw4tkcx2w4vaxkwyzjbtsghadmnwpq6yaqfi7qd.onion	".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "electrumx.tamami-foundation.org".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
            discovery.add_default_server(
                "electrumx1.monacoin.ninja".into(),
                vec![Service::Tcp(50001), Service::Ssl(50002)],
            );
        }
        Network::MonacoinTestnet => {
            discovery.add_default_server(
                "electrumx2.testnet.monacoin.ninja".into(),
                vec![Service::Tcp(51001), Service::Ssl(51002)],
            );
            discovery.add_default_server(
                "electrumx1.testnet.monacoin.nl".into(),
                vec![Service::Tcp(51001), Service::Ssl(51002)],
            );
        }

        _ => (),
    }
}
