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

from zenoh_flow.interfaces import Source
from zenoh_flow import DataSender
from typing import Any, Dict, Callable
import time
import asyncio

class MyState:
    def __init__(self, configuration):
        self.interval = 1
        self.size = 64
        if configuration is not None and configuration.get('msgs',None) is not None:
            self.interval = 1/int(configuration['msgs'])
            self.msgs = int(configuration['msgs'])
        if configuration is not None and configuration.get('size', None) is not None:
            self.size = int(configuration['size'])

        self.payload = b'0' * self.size

class MySrc(Source):

    def setup(self, configuration: Dict[str, Any], outputs: Dict[str, DataSender]) -> Callable[[], Any]:
        state = MyState(configuration)
        output = outputs.get('Value', None)
        return lambda: create_data(output, state)

    def finalize(self) -> None:
        return None


async def create_data(output, state):
    await asyncio.sleep(state.interval)
    data = int_to_bytes(time.time_ns()) + state.payload
    await output.send(data)
    return None


def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, 'big')

def register():
    return MySrc