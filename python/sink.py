#
# Copyright (c) 2022 ZettaScale Technology
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
#

from zenoh_flow.interfaces import Sink
from zenoh_flow import DataReceiver
from typing import Dict, Any, Callable
import time


class MyState:
    def __init__(self, configuration):
        self.interval = 1
        self.msgs = 1
        self.size = 64
        if configuration is not None and configuration.get('msgs',None) is not None:
            self.interval = 1/int(configuration['msgs'])
            self.msgs = int(configuration['msgs'])
        if configuration is not None and configuration.get('size', None) is not None:
            self.size = int(configuration['size'])
        print("Config {configuration}")


class MySink(Sink):

    def finalize(self):
        return None

    def setup(self, configuration: Dict[str, Any], inputs: Dict[str, DataReceiver]) -> Callable[[], Any]:
        state = MyState(configuration)
        in_stream = inputs.get('Value', None)
        return lambda: run(in_stream, state)

async def run(in_stream, state):
        data_msg = await in_stream.recv()
        data = data_msg.data
        now = time.time_ns()
        ts_value = data[:8]
        value = bytes_to_int(ts_value)
        elapsed = now - value
        print(f'zenoh-flow-python,{state.size},{state.msgs},{elapsed},ns')
        return None

def bytes_to_int(xbytes: bytes) -> int:
    return int.from_bytes(xbytes, 'big')

def register():
    return MySink