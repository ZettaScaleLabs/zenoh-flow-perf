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

import zenoh
import time
import struct
from zenoh_flow import Sink


class MyState:
    def __init__(self, configuration):
        self.interval = 1
        self.size = 64
        if configuration['msgs'] is not None:
            self.interval = 1/int(configuration['msgs'])
        if configuration['size'] is not None:
            self.size = int(configuration['size'])
        self.key_expr = '/test/latency/zf/pong'
        self.zenoh = zenoh.open(None)

    def close(self):
        self.zenoh.close()

class MySrc(Sink):
    def initialize(self, configuration):
        return MyState(configuration)

    def finalize(self, state):
        return state.close()

    def run(self, _ctx, state, data):
        now = time.time_ns()
        value = bytes_to_int(data.data)
        elapsed = now - value

        print(f'zenoh-flow-python,single,latency,1,{state.size},0,{elapsed},ns')

        state.zenoh.put(state.key_expr, int_to_bytes(now))

        return None




def int_to_bytes(x: int) -> bytes:
    return struct.pack("<Q", x)

def bytes_to_int(x: bytes) -> int:
    return struct.unpack('<Q', x)[0]

def register():
    return MySrc