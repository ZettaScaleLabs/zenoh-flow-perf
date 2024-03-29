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
from zenoh_flow import Output
from zenoh_flow.types import Context
from typing import Any, Dict
import time
import asyncio


class MySrc(Source):

    def __init__(
        self,
        context: Context,
        configuration: Dict[str, Any],
        outputs: Dict[str, Output],
    ):

        configuration = {} if configuration is None else configuration

        self.interval = 1
        self.size = 64
        if configuration.get('msgs', None) is not None:
            self.interval = 1/int(configuration['msgs'])
            self.msgs = int(configuration['msgs'])
        if configuration.get('size', None) is not None:
            self.size = int(configuration['size'])

        self.payload = b'0' * self.size

        self.output = outputs.get('Value', None)

    def finalize(self) -> None:
        return None

    async def iteration(self) -> None:
        await asyncio.sleep(self.interval)
        data = int_to_bytes(time.time_ns()) + self.payload
        await self.output.send(data)
        return None


def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, 'big')


def register():
    return MySrc
