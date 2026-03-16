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

#include "src/feo/cpp-src/mw_com_gen.h"
#include "score/mw/com/impl/rust/com-api/com-api-ffi-lola/registry_bridge_macro.h"

BEGIN_EXPORT_MW_COM_INTERFACE(FeoSignalInterface, ::score::feo::com::FeoSignalProxy, ::score::feo::com::FeoSignalSkeleton)
EXPORT_MW_COM_EVENT(::score::feo::com::MwComSignal, signal)
END_EXPORT_MW_COM_INTERFACE()

EXPORT_MW_COM_TYPE(MwComSignal, ::score::feo::com::MwComSignal)
