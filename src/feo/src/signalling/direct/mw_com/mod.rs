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

//! Elements for com-based signalling

pub(crate) mod mw_com_gen;
pub(crate) mod scheduler_connector;
pub(crate) mod worker_connector;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use com_api::{CommData, LolaRuntimeImpl, Reloc, Runtime, SampleContainer, Subscriber, Subscription};
use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::pin::Pin;
use feo_com::interface::FeoComData;
use futures::stream::SelectAll;
use futures::task::Context;
use futures::task::Poll;
use futures::Future;
use futures::Stream;
use score_log::ScoreDebug;

pub type MwComPublisher<T> = <LolaRuntimeImpl as Runtime>::Publisher<T>;
pub type MwComSubscription<T> =
    <<LolaRuntimeImpl as Runtime>::Subscriber<T> as Subscriber<T, LolaRuntimeImpl>>::Subscription;
pub type MwComSample<'a, T> = <MwComSubscription<T> as Subscription<T, LolaRuntimeImpl>>::Sample<'a>;

/// Protocol-specific signal type
///
/// This type wraps the core [Signal] type and adds signal variants
/// which are only needed by this particular implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ScoreDebug)]
#[repr(C, u8)]
pub(crate) enum MwComSignal {
    /// Core [Signal] as known to schedulers and workers
    Core(crate::signalling::common::signals::Signal),
    /// Hello signal announcing the presence of an activity with [ActivityId]
    ActivityHello(crate::ids::ActivityId),
    /// Hello signal confirmation
    ActivityHelloAcquired,
}

// SAFETY: safe to relocate
unsafe impl Reloc for MwComSignal {}

impl CommData for MwComSignal {
    const ID: &'static str = "MwComSignal";
}

type MwComSubscriptions<T> = SelectAll<MwComSubscriptionStream<T>>;
type MwComSubscriptionReceiveFut<T> = dyn Future<Output = com_api::Result<SampleContainer<MwComSample<'static, T>>>>;

// TODO: This is stub implementation of Stream API for an mw com Subscription, replace it with a proper
// one as soon as it's available in communication
// SAFETY: the implementation is almost certainly unsound
struct MwComSubscriptionStream<T: FeoComData> {
    /// Subscription
    pub(crate) subscription: Arc<UnsafeCell<MwComSubscription<T>>>,
    /// Sample container
    pub(crate) sample_container: Option<SampleContainer<MwComSample<'static, T>>>,
    pub(crate) future: RefCell<Option<Pin<Box<MwComSubscriptionReceiveFut<T>>>>>,
    /// Received sample data
    pub(crate) received: RefCell<Vec<MwComSample<'static, T>>>,
}

impl<T: FeoComData> MwComSubscriptionStream<T> {
    pub fn new(subscription: MwComSubscription<T>, capacity: usize) -> Self {
        Self {
            subscription: Arc::new(UnsafeCell::new(subscription)),
            sample_container: Some(SampleContainer::new(capacity)),
            future: RefCell::new(None),
            received: RefCell::new(Vec::with_capacity(capacity)),
        }
    }
}

impl<T: FeoComData> Stream for MwComSubscriptionStream<T> {
    type Item = MwComSample<'static, T>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<<Self as Stream>::Item>> {
        loop {
            if !self.received.borrow().is_empty() {
                return Poll::Ready(Some(self.received.borrow_mut().pop().unwrap()));
            }
            if self.future.borrow().is_none() {
                let sample_container = self
                    .sample_container
                    .take()
                    .expect("concurrent receive calls detected on the same subscription");
                self.future.replace(Some(Box::pin(
                    // SAFETY: the future is not polled if Stream gets out of scope and this is the only
                    // place we use subscription
                    unsafe { &*self.subscription.get() }.receive(
                        sample_container,
                        1,
                        self.received.borrow().capacity(),
                    ),
                )));
            }
            let mut sample_container = match Future::poll(self.future.borrow_mut().as_mut().unwrap().as_mut(), ctx) {
                Poll::Ready(sample_container) => sample_container.unwrap(),
                Poll::Pending => return Poll::Pending,
            };
            assert!(sample_container.sample_count() > 0);
            assert!(
                sample_container.sample_count() + self.received.borrow().len() <= self.received.borrow().capacity(),
                "must not allocate, configuration error"
            );
            for _ in 0..sample_container.sample_count() {
                // keep ordering
                self.received
                    .borrow_mut()
                    .insert(0, sample_container.pop_front().unwrap());
            }
            self.sample_container = Some(sample_container);
            self.future.replace(None);
        }
    }
}
