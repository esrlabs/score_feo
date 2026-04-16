// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use feo_time::Duration;
use std::collections::{HashMap, HashSet};

use crate::alloc::string::ToString;
use crate::ids::WorkerId;
use crate::signalling::direct::mw_com::mw_com_gen::FeoSignalInterface;
use crate::signalling::direct::mw_com::worker_connector::create_consumer;
use crate::signalling::direct::mw_com::worker_connector::create_producer;
use crate::signalling::direct::mw_com::MwComPublisher;
use crate::signalling::direct::mw_com::MwComSignal;
use crate::signalling::direct::mw_com::MwComSubscriptionStream;
use crate::signalling::direct::mw_com::MwComSubscriptions;
use crate::timestamp::sync_info;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use com_api::LolaRuntimeImpl;
use com_api::Publisher;
use com_api::Subscriber;
use core::mem;
use futures::stream::select_all;
use futures::StreamExt;
use score_log::{debug, warn};

use crate::{
    error::Error,
    ids::{ActivityId, AgentId},
    signalling::common::{interface::ConnectScheduler, signals::Signal},
};

type WorkerWithActivities = (WorkerId, Vec<ActivityId>);

pub struct MwComSchedulerConnector {
    agent_assignments: Vec<(AgentId, Vec<WorkerWithActivities>)>,
    activities_workers: HashMap<ActivityId, WorkerId>,
    agents_input: MwComSubscriptions<MwComSignal>,
    worker_publishers: HashMap<WorkerId, MwComPublisher<MwComSignal>>,
}

impl MwComSchedulerConnector {
    /// Create a new instance
    pub(crate) fn new(
        _self_agent_id: AgentId,
        agent_assignments: Vec<(AgentId, Vec<WorkerWithActivities>)>,
        runtime: &LolaRuntimeImpl,
    ) -> Self {
        let mut agent_inputs = vec![];
        let mut worker_publishers = HashMap::new();
        let mut activities_workers = HashMap::new();

        debug!("Creating agent consumers for scheduler...");
        for agent_id in agent_assignments.iter().map(|(a, _)| *a) {
            let subscription_stream = MwComSubscriptionStream::new(
                create_consumer::<FeoSignalInterface>(
                    runtime,
                    &format!("/feo/com/Agent{}ToScheduler", agent_id.to_string().replace('-', "")),
                )
                .signal
                .subscribe(10)
                .expect("Failed to subscribe"),
                10,
            );
            agent_inputs.push(subscription_stream);
        }

        debug!("Creating worker producers for scheduler...");
        // Worker skeletons
        for (worker_id, activity_ids) in agent_assignments.iter().flat_map(|(_, v)| v.iter()) {
            for activity_id in activity_ids {
                activities_workers.insert(*activity_id, *worker_id);
            }
            let producer =
                create_producer::<FeoSignalInterface>(runtime, &format!("/feo/com/SchedulerToWorker{}", worker_id));
            worker_publishers.insert(*worker_id, producer.signal);
        }

        Self {
            activities_workers,
            agent_assignments,
            agents_input: select_all(agent_inputs),
            worker_publishers,
        }
    }
}

impl ConnectScheduler for MwComSchedulerConnector {
    fn connect_remotes(&mut self) -> Result<(), Error> {
        use futures::StreamExt;

        let mut missing_activities: HashSet<ActivityId> = self
            .agent_assignments
            .iter()
            .flat_map(|(_, v)| v.iter().flat_map(|(_, a)| a.iter().copied()))
            .collect();

        debug!("Connecting remotes of scheduler...");
        let worker_publishers = mem::take(&mut self.worker_publishers);
        let activities_workers = mem::take(&mut self.activities_workers);
        {
            while !missing_activities.is_empty() {
                debug!("Waiting for a signal from agents...");
                std::dbg!(&missing_activities);
                if let Some(signal) = futures::executor::block_on(self.agents_input.next()) {
                    debug!("Got signal {:?}", *signal);
                    match *signal {
                        MwComSignal::ActivityHello(activity_id) => {
                            missing_activities.remove(&activity_id);
                            worker_publishers
                                .get(activities_workers.get(&activity_id).expect("No worker for activity"))
                                .expect("No skeleton for worker")
                                .send(MwComSignal::ActivityHelloAcquired)
                                .expect("Failed sending event");
                        },
                        other => {
                            warn!("received unexpected signal {:?}", other);
                        },
                    }
                }
            }
        }
        self.worker_publishers = worker_publishers;
        self.activities_workers = activities_workers;
        Ok(())
    }

    fn sync_time(&mut self) -> Result<(), Error> {
        debug!("Syncing time...");
        let signal = MwComSignal::Core(Signal::StartupSync(sync_info()));

        // Send startup time to all workers
        for publisher in self.worker_publishers.values_mut() {
            publisher.send(signal).expect("Failed sending event");
        }

        Ok(())
    }

    fn receive(&mut self, _timeout: Duration) -> Result<Option<Signal>, Error> {
        // FIXME: add timeout when it's supported by MW COM API (tokio::time::timeout(timeout.into(), ...))
        debug!("Waiting for a signal from agents...");
        loop {
            let signal = futures::executor::block_on(self.agents_input.next()).expect("Can't receive signal");
            // let (signal) = signal else { return Ok(None) };
            if let MwComSignal::Core(signal) = *signal {
                return Ok(Some(signal));
            }
        }
    }

    fn send_to_activity(&mut self, activity_id: ActivityId, signal: &Signal) -> Result<(), Error> {
        self.worker_publishers
            .get_mut(
                self.activities_workers
                    .get(&activity_id)
                    .expect("No worker for activity"),
            )
            .expect("No skeleton for activity")
            .send(MwComSignal::Core(*signal))
            .expect("Failed sending event");
        Ok(())
    }

    fn get_connected_agent_ids(&self) -> alloc::vec::Vec<AgentId> {
        // In direct mode, there are no remote agents
        alloc::vec![]
    }

    fn broadcast_terminate(&mut self, _signal: &Signal) -> Result<(), Error> {
        // In direct mode, all workers are local threads. Broadcast to them.
        let protocol_signal = MwComSignal::Core(Signal::Terminate(crate::timestamp::timestamp()));
        // Send startup time to all workers
        for publisher in self.worker_publishers.values_mut() {
            publisher.send(protocol_signal).expect("Failed sending event");
        }
        Ok(())
    }
}
