/********************************************************************************
 * Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#ifndef SCORE_FEO_DATATYPE_H
#define SCORE_FEO_DATATYPE_H

#include "score/mw/com/types.h"

namespace score::feo::com
{
struct Nanoseconds {
    std::uint32_t val;
};

struct Duration {
    std::uint64_t secs;
    Nanoseconds nanos; // Always 0 <= nanos < NANOS_PER_SEC
};

struct Timestamp {
    Duration value;
};

struct SyncInfo {
    Duration since_epoch;
};

struct ActivityId {
    std::uint64_t id;
};

struct GenericId {
    std::uint64_t id;
};

struct AgentId {
    std::uint64_t id;
};

struct ChannelId {
    std::uint64_t id;
};

enum class SignalTag: std::uint8_t { StartupSync, Startup, Shutdown, Step, Ready, TaskChainStart, TaskChainEnd, Terminate, TerminateAcquired };
struct SignalStartupSync { SignalTag tag; SyncInfo sync_info; };
struct SignalStartup { SignalTag tag; ActivityId activity_id; Timestamp timestamp; };
struct SignalShutdown { SignalTag tag; ActivityId activity_id; Timestamp timestamp; };
struct SignalStep { SignalTag tag; ActivityId activity_id; Timestamp timestamp; };
struct SignalReady { SignalTag tag; ActivityId activity_id; Timestamp timestamp; };
struct SignalTaskChainStart { SignalTag tag; Timestamp timestamp; };
struct SignalTaskChainEnd { SignalTag tag; Timestamp timestamp; };
struct SignalTerminate { SignalTag tag; Timestamp timestamp; };
struct SignalTerminateAcquired { SignalTag tag; AgentId agent_id; };

union Signal {
    SignalStartupSync sync;
    SignalStartup startup;
    SignalShutdown shutdown;
    SignalStep step;
    SignalReady ready;
    SignalTaskChainStart task_chain_start;
    SignalTaskChainEnd task_chain_end;
    SignalTerminate terminate;
    SignalTerminateAcquired terminate_acquired;
};

enum class MwComSignalTag: std::uint8_t { Core, ActivityHello, ChannelHello };
struct MwComSignalCore { MwComSignalTag tag; Signal signal; };
struct MwComSignalActivityHello { MwComSignalTag tag; ActivityId activity_id; };
struct MwComSignalActivityHelloAcquired { MwComSignalTag tag; };

union MwComSignal {
    MwComSignalCore core;
    MwComSignalActivityHello activity_hello;
    MwComSignalActivityHelloAcquired activity_hello_acquired;
};

template <typename Trait>
class FeoSignalInterface : public Trait::Base
{
  public:
    using Trait::Base::Base;
    typename Trait::template Event<MwComSignal> signal{*this, "signal"};
};
using FeoSignalProxy = mw::com::AsProxy<FeoSignalInterface>;
using FeoSignalSkeleton = mw::com::AsSkeleton<FeoSignalInterface>;

}  // namespace score::feo::com
#endif  // SCORE_FEO_DATATYPE_H
