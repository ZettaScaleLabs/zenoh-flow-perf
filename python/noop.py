##
## Copyright (c) 2017, 2021 ADLINK Technology Inc.
##
## This program and the accompanying materials are made available under the
## terms of the Eclipse Public License 2.0 which is available at
## http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
## which is available at https://www.apache.org/licenses/LICENSE-2.0.
##
## SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
##
## Contributors:
##   ADLINK zenoh team, <zenoh@adlink-labs.tech>
##

from zenoh_flow import Inputs, Operator, Outputs

class MyOp(Operator):
    def initialize(self, configuration):
        return None

    def finalize(self, state):
        return None

    def input_rule(self, _ctx, state, tokens):
        return True

    def output_rule(self, _ctx, _state, outputs, _deadline_miss):
        return outputs

    def run(self, _ctx, _state, inputs):
        # Getting the inputs
        data = inputs.get('Data').data
        outputs = {'Data' : data}
        return outputs


def register():
    return MyOp