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

use crate::*;

use com_api::{interface, CommData, ProviderInfo, Publisher, Subscriber};

impl CommData for CameraImage {
    const ID: &'static str = "CameraImage";
}

interface!(
    interface Camera, {
        Id = "CameraInterface",
        image: Event<CameraImage>,
     }
);

impl CommData for RadarScan {
    const ID: &'static str = "RadarScan";
}

interface!(
    interface Radar, {
        Id = "RadarInterface",
        scan: Event<RadarScan>,
     }
);

impl CommData for Scene {
    const ID: &'static str = "Scene";
}

interface!(
    interface NeuralNet, {
        Id = "NeuralNetInterface",
        scene: Event<Scene>,
     }
);

impl CommData for BrakeInstruction {
    const ID: &'static str = "BrakeInstruction";
}

interface!(
    interface BrakeController, {
        Id = "BrakeControllerInterface",
        brake_instruction: Event<BrakeInstruction>,
     }
);

impl CommData for Steering {
    const ID: &'static str = "Steering";
}

interface!(
    interface SteeringController, {
        Id = "SteeringControllerInterface",
        steering: Event<Steering>,
     }
);
