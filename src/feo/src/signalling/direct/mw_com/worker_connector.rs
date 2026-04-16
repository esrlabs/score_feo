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

use crate::alloc::string::ToString;
use crate::error::Error;
use crate::ids::ActivityId;
use crate::ids::AgentId;
use crate::ids::WorkerId;
use crate::signalling::common::interface::ConnectWorker;
use crate::signalling::common::signals::Signal;
use crate::signalling::direct::mw_com::mw_com_gen::FeoSignalInterface;
use crate::signalling::direct::mw_com::MwComPublisher;
use crate::signalling::direct::mw_com::MwComSignal;
use crate::signalling::direct::mw_com::MwComSubscriptionStream;
use alloc::format;
use alloc::sync::Arc;
use alloc::vec::Vec;
use com_api::Publisher;
use com_api::{
    Builder, FindServiceSpecifier, InstanceSpecifier, Interface, LolaRuntimeImpl, Producer, Runtime, ServiceDiscovery,
    Subscriber,
};
use feo_time::Duration;
use futures::StreamExt;
use score_log::{debug, trace};
use std::sync::Barrier;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread::sleep;

type AgentPublisher = MwComPublisher<MwComSignal>;

/// MwCom connector for a worker
pub(crate) struct MwComWorkerConnector {
    worker_id: WorkerId,
    barrier: Arc<Barrier>,
    activity_ids: Vec<ActivityId>,
    agent_output: Arc<Mutex<AgentPublisher>>,
    scheduler_input: Option<MwComSubscriptionStream<MwComSignal>>,
    runtime: &'static LolaRuntimeImpl,
}

impl MwComWorkerConnector {
    pub fn new(
        barrier: Arc<Barrier>,
        activity_ids: Vec<ActivityId>,
        worker_id: WorkerId,
        agent_output: Arc<Mutex<AgentPublisher>>,
        runtime: &'static LolaRuntimeImpl,
    ) -> Self {
        Self {
            worker_id,
            barrier,
            activity_ids,
            agent_output,
            scheduler_input: Default::default(),
            runtime,
        }
    }

    pub fn agent_output(&self) -> MutexGuard<'_, AgentPublisher> {
        self.agent_output.lock().unwrap()
    }

    pub(crate) fn send_signal(&self, signal: MwComSignal) {
        self.agent_output().send(signal).expect("Failed sending event");
    }
}

impl ConnectWorker for MwComWorkerConnector {
    fn connect_remote(&mut self) -> Result<(), Error> {
        // Wait for other subscriptions to initialize
        debug!("Waiting for other subscriptions to initialize (we need to create the agent subscription after all the worker producers are initialized)...");
        self.barrier.wait();
        debug!("Workers' producers are initialized, so we can create subscriptions...");

        let consumer = create_consumer::<FeoSignalInterface>(
            self.runtime,
            // allocates but only on initialization
            &format!("/feo/com/SchedulerToWorker{}", self.worker_id),
        );
        self.scheduler_input = Some(MwComSubscriptionStream::new(
            consumer.signal.subscribe(10).expect("Failed to subscribe"),
            10,
        ));

        // Hello
        for activity_id in &self.activity_ids {
            debug!("Sending Hello for {:?}...", activity_id);
            self.send_signal(MwComSignal::ActivityHello(*activity_id));
            let scheduler_input = self
                .scheduler_input
                .as_mut()
                .expect("Scheduler proxy is not initialized");
            debug!("Waiting for a signal from scheduler (activity {})...", activity_id);
            // FIXME: add timeout when it's supported by MW COM API (tokio::time::timeout(Duration::from_secs(10).into(), ...))
            let signal = futures::executor::block_on(scheduler_input.next()).unwrap();
            debug!("Got a signal from scheduler (activity {})...", activity_id);
            match *signal {
                MwComSignal::ActivityHelloAcquired => {
                    debug!("ActivityHello confirmation received");
                    continue;
                },
                s => panic!("Unexpected signal {:?}", s),
            }
            // debug!(
            //     "ActivityHello confirmation for {:?} has not been received in 10 seconds",
            //     activity_id
            // );
        }
        Ok(())
    }

    fn receive(&mut self, _timeout: Duration) -> Result<Option<Signal>, Error> {
        let scheduler_input = self
            .scheduler_input
            .as_mut()
            .expect("Scheduler proxy is not initialized");
        // FIXME: add timeout when it's supported by MW COM API (tokio::time::timeout(timeout.into(), ...))
        trace!("Waiting for a signal from scheduler...");
        let signal = futures::executor::block_on(scheduler_input.next()).unwrap();
        // FIXME: restore timeout handling when com api supports it
        // else {
        //     // timeout or no message
        //     return Ok(None);
        // };
        if let MwComSignal::Core(signal) = *signal {
            trace!("Got signal from scheduler {:?}", signal);
            Ok(Some(signal))
        } else {
            Err(Error::UnexpectedProtocolSignal)
        }
    }

    fn send_to_scheduler(&mut self, signal: &Signal) -> Result<(), Error> {
        self.send_signal(MwComSignal::Core(*signal));
        Ok(())
    }
}

pub(crate) fn agent_output(runtime: &LolaRuntimeImpl, agent_id: AgentId) -> Arc<Mutex<AgentPublisher>> {
    trace!("Creating producer for agent output to scheduler...");
    let instance_specifier_str = format!("/feo/com/Agent{}ToScheduler", agent_id.to_string().replace('-', ""));
    let skeleton = create_producer::<FeoSignalInterface>(runtime, &instance_specifier_str);
    Arc::new(Mutex::new(skeleton.signal))
}

pub(crate) fn create_producer<I: Interface>(
    runtime: &LolaRuntimeImpl,
    topic: &str,
) -> <<I as Interface>::Producer<LolaRuntimeImpl> as Producer<LolaRuntimeImpl>>::OfferedProducer {
    trace!("Creating mw com producer for {}...", topic);
    let service_id = InstanceSpecifier::new(topic).expect("failed to create InstanceSpecifier");
    let producer_builder = runtime.producer_builder::<I>(service_id);
    let producer = producer_builder.build().unwrap();
    producer
        .offer()
        .map_err(|e| format!("can't offer an mw com producer for {topic}: {e:?}"))
        .unwrap()
}

pub(crate) fn create_consumer<I: Interface + Send>(
    runtime: &LolaRuntimeImpl,
    topic: &str,
) -> <I as Interface>::Consumer<LolaRuntimeImpl> {
    trace!("Creating mw com consumer for {}...", topic);
    let mut tries = 0;
    loop {
        let service_id = InstanceSpecifier::new(topic).expect("Failed to create InstanceSpecifier");
        let consumer_discovery = runtime.find_service::<I>(FindServiceSpecifier::Specific(service_id));
        let available_service_instances = consumer_discovery.get_available_instances().unwrap();

        if available_service_instances.is_empty() {
            sleep(Duration::from_millis(100).into());
            tries += 1;
            if tries > 100 {
                break;
            }
            continue;
        }

        // Select service instance at specific handle_index
        let handle_index = 0; // or any index you need from vector of instances
        let consumer_builder = available_service_instances.into_iter().nth(handle_index).unwrap();

        return consumer_builder.build().unwrap();
    }
    panic!("Can't create consumer for {topic} (timed out)")
}
