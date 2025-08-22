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

use feo::cpp_activity;

// Create glue code for C++ activities
// Note: Currently this only works for activities having no arguments to their C++ step function
//       (as is to be expected once the com layer is interoperable between Rust and C++)
cpp_activity!(lane_assist, "cpp_activities");
cpp_activity!(trajectory_visualizer, "cpp_activities");
