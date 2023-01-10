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
from zenoh_flow import Input
from zenoh_flow.types import Context
from typing import Dict, Any
import time


class MySink(Sink):

    def finalize(self):
        return None

    def __init__(
        self,
        context: Context,
        configuration: Dict[str, Any],
        inputs: Dict[str, Input],
    ):
        configuration = {} if configuration is None else configuration

        self.interval = 1
        self.msgs = 1
        self.size = 64
        if configuration.get('msgs', None) is not None:
            self.interval = 1/int(configuration['msgs'])
            self.msgs = int(configuration['msgs'])
        if configuration.get('size', None) is not None:
            self.size = int(configuration['size'])

        self.in_stream = inputs.get('Value', None)

    async def iteration(self):
        data_msg = await self.in_stream.recv()
        data = data_msg.data
        now = time.time_ns()
        ts_value = data[:8]
        value = bytes_to_int(ts_value)
        elapsed = now - value
        print(f'zenoh-flow-python,{self.size},{self.msgs},{elapsed},ns')
        return None


def bytes_to_int(xbytes: bytes) -> int:
    return int.from_bytes(xbytes, 'big')


def register():
    return MySink
