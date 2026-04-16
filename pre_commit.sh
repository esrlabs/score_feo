#!/usr/bin/env bash

# *******************************************************************************
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
# *******************************************************************************

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

buildifier -r $SCRIPT_DIR
bazelisk mod tidy
#bazelisk mod deps --lockfile_mode=update
bazelisk run --lockfile_mode=error //:format.fix
bazelisk run --lockfile_mode=error //:format.check_Rust_with_rustfmt
bazelisk run --lockfile_mode=error //:copyright.check
bazelisk build --lockfile_mode=error --config=lint-rust //...
bazelisk build --lockfile_mode=error --config=x86_64-linux //...
bazelisk test --lockfile_mode=error --config=x86_64-linux //...
