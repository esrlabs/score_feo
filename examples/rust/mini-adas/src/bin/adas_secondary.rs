/********************************************************************************
 * Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

#[cfg(not(feature = "com_mw"))]
use adas::config::{agent_assignments_ids, topic_dependencies, COM_BACKEND};
#[cfg(not(feature = "com_mw"))]
use feo::agent::com_init::initialize_com_secondary;
#[cfg(not(feature = "com_mw"))]
use feo::ids::ActivityId;
use score_log::{info, LevelFilter};
#[cfg(not(feature = "com_mw"))]
use std::collections::HashSet;
use stdout_logger::StdoutLoggerBuilder;

#[cfg(any(
    feature = "signalling_direct_tcp",
    feature = "signalling_direct_unix",
    feature = "signalling_direct_mw_com"
))]
fn main() {
    use adas::config::agent_assignments;
    use adas::config::init_mw_com_runtime;
    #[cfg(feature = "signalling_direct_unix")]
    use adas::config::socket_paths;
    #[cfg(feature = "signalling_direct_tcp")]
    use adas::config::BIND_ADDR;
    use feo::agent::direct::secondary::{Secondary, SecondaryConfig};
    use feo::agent::NodeAddress;
    use feo_time::Duration;
    use params::Params;

    init_logging();

    feo_tracing::init(feo_tracing::LevelFilter::TRACE);

    let params = Params::from_args();
    info!("Starting agent {}", params.agent_id);

    let config = SecondaryConfig {
        id: params.agent_id,
        worker_assignments: agent_assignments().remove(&params.agent_id).unwrap(),
        timeout: Duration::from_secs(1),
        #[cfg(feature = "signalling_direct_tcp")]
        endpoint: NodeAddress::Tcp(BIND_ADDR),
        #[cfg(feature = "signalling_direct_unix")]
        endpoint: NodeAddress::UnixSocket(socket_paths().0),
        #[cfg(feature = "signalling_direct_mw_com")]
        endpoint: NodeAddress::MwCom,
    };

    // determine set of activity ids belonging to this agent
    #[cfg(not(feature = "com_mw"))]
    let local_activities: HashSet<ActivityId> = agent_assignments_ids()
        .remove(&params.agent_id)
        .unwrap()
        .iter()
        .flat_map(|(_, acts)| acts.iter())
        .copied()
        .collect();

    // Initialize MW COM
    let runtime = init_mw_com_runtime(params.agent_id);

    // Initialize topics. Do not drop.
    #[cfg(not(feature = "com_mw"))]
    let _topic_guards = initialize_com_secondary(COM_BACKEND, topic_dependencies(), &local_activities);

    let secondary = Secondary::new(config, runtime);
    secondary.run();
}

#[cfg(feature = "signalling_relayed_tcp")]
fn main() {
    use adas::config::agent_assignments;
    #[cfg(feature = "com_mw")]
    use adas::config::init_mw_com_runtime;
    use adas::config::{BIND_ADDR, BIND_ADDR2};
    use feo::agent::relayed::secondary::{Secondary, SecondaryConfig};
    use feo::agent::NodeAddress;
    use feo_time::Duration;
    use params::Params;

    init_logging();

    feo_tracing::init(feo_tracing::LevelFilter::TRACE);

    let params = Params::from_args();
    info!("Starting agent {}", params.agent_id);

    let config = SecondaryConfig {
        id: params.agent_id,
        worker_assignments: agent_assignments().remove(&params.agent_id).unwrap(),
        timeout: Duration::from_secs(10),
        bind_address_senders: NodeAddress::Tcp(BIND_ADDR),
        bind_address_receivers: NodeAddress::Tcp(BIND_ADDR2),
    };

    // determine set of activity ids belonging to this agent
    #[cfg(not(feature = "com_mw"))]
    let local_activities: HashSet<ActivityId> = agent_assignments_ids()
        .remove(&params.agent_id)
        .unwrap()
        .iter()
        .flat_map(|(_, acts)| acts.iter())
        .copied()
        .collect();

    // Initialize MW COM
    #[cfg(feature = "com_mw")]
    init_mw_com_runtime(params.agent_id);

    // Initialize topics. Do not drop.
    #[cfg(not(feature = "com_mw"))]
    let _topic_guards = initialize_com_secondary(COM_BACKEND, topic_dependencies(), &local_activities);

    let secondary = Secondary::new(config);
    secondary.run();
}

#[cfg(feature = "signalling_relayed_unix")]
fn main() {
    use adas::config::agent_assignments;
    #[cfg(feature = "com_mw")]
    use adas::config::init_mw_com_runtime;
    use adas::config::socket_paths;
    use feo::agent::relayed::secondary::{Secondary, SecondaryConfig};
    use feo::agent::NodeAddress;
    use feo_time::Duration;
    use params::Params;

    init_logging();
    feo_tracing::init(feo_tracing::LevelFilter::TRACE);

    let params = Params::from_args();
    info!("Starting agent {}", params.agent_id);

    let config = SecondaryConfig {
        id: params.agent_id,
        worker_assignments: agent_assignments().remove(&params.agent_id).unwrap(),
        timeout: Duration::from_secs(10),
        bind_address_senders: NodeAddress::UnixSocket(socket_paths().0),
        bind_address_receivers: NodeAddress::UnixSocket(socket_paths().1),
    };

    // determine set of activity ids belonging to this agent
    #[cfg(not(feature = "com_mw"))]
    let local_activities: HashSet<ActivityId> = agent_assignments_ids()
        .remove(&params.agent_id)
        .unwrap()
        .iter()
        .flat_map(|(_, acts)| acts.iter())
        .copied()
        .collect();

    // Initialize MW COM
    #[cfg(feature = "com_mw")]
    init_mw_com_runtime(params.agent_id);

    // Initialize topics. Do not drop.
    #[cfg(not(feature = "com_mw"))]
    let _topic_guards = initialize_com_secondary(COM_BACKEND, topic_dependencies(), &local_activities);

    let secondary = Secondary::new(config);
    secondary.run();
}

#[cfg(feature = "signalling_direct_mpsc")]
fn main() {
    panic!("Secondaries are not supported with this feature flag");
}

#[cfg(not(feature = "signalling_direct_mpsc"))]
mod params {
    use feo::ids::AgentId;
    /// Parameters of the secondary
    pub struct Params {
        /// Secondary agent ID
        pub agent_id: AgentId,
    }

    impl Params {
        const SECONDARY_IDS: [AgentId; 2] = [AgentId::new(101), AgentId::new(102)];
        pub fn from_args() -> Self {
            let args: Vec<String> = std::env::args().collect();

            let secondary_index = args
                .get(1)
                .and_then(|x| x.parse::<usize>().ok())
                .expect("invalid secondary index");

            let agent_id = Self::SECONDARY_IDS
                .get(secondary_index - 1)
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "secondary index must be in the range 1 ... {}",
                        Self::SECONDARY_IDS.len()
                    )
                });

            Self { agent_id }
        }
    }
}

fn init_logging() {
    StdoutLoggerBuilder::new()
        .context("adas-secondary")
        .show_module(false)
        .show_file(false)
        .show_line(false)
        .log_level(LevelFilter::Trace)
        .set_as_default_logger();
}
