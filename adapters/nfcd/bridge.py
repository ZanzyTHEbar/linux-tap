#!/usr/bin/env python3
"""Development transport bridge. No wallet keys, payment approval, or chain access.

Exports the upstream nfcd LocalHostApp interface and delegates bounded APDUs to
the Rust state machine. Requires a working nfcd hardware adapter and D-Bus policy.
The output is an UNVERIFIED invoice; acknowledgement never means payment.
"""
import argparse
import asyncio
import signal
import subprocess
from pathlib import Path

from dbus_next import BusType
from dbus_next.aio import MessageBus
from dbus_next.service import ServiceInterface, method

AID = bytes.fromhex('f04c54415000')
PATH = '/org/linuxtap/InvoiceReceiver'

class HostApp(ServiceInterface):
    def __init__(self, binary, output):
        super().__init__('org.sailfishos.nfc.LocalHostApp')
        self.child = subprocess.Popen([binary, 'apdu-stdio', '--out', output],
                                      stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                      text=True, bufsize=1)

    def exchange(self, command):
        # Rust also enforces the line length and APDU/message bounds.
        self.child.stdin.write(command + '\n')
        self.child.stdin.flush()
        response = self.child.stdout.readline(1025)
        if not response.endswith('\n') or len(response) > 1024:
            raise RuntimeError('Rust receiver failed or returned an oversized response')
        return bytes.fromhex(response.strip())

    def reset(self):
        self.exchange('RESET')

    def select(self):
        self.exchange((bytes([0, 0xa4, 4, 0, len(AID)]) + AID).hex())

    @method()
    def GetInterfaceVersion(self) -> 'i':
        return 1

    @method()
    def Start(self, host: 'o'):
        self.reset()

    @method()
    def Restart(self, host: 'o'):
        self.reset()

    @method()
    def Stop(self, path: 'o'):
        self.reset()

    @method()
    def ImplicitSelect(self, host: 'o'):
        self.select()

    @method()
    def Select(self, host: 'o'):
        self.select()

    @method()
    def Deselect(self, path: 'o'):
        self.reset()

    @method()
    def Process(self, host: 'o', CLA: 'y', INS: 'y', P1: 'y', P2: 'y', data: 'ay', Le: 'u') -> 'ayyyu':
        if len(data) > 255 or Le > 256:
            return [b'', 0x67, 0, 0]
        response = self.exchange((bytes([CLA, INS, P1, P2, len(data)]) + data).hex())
        if len(response) < 2:
            return [b'', 0x6f, 0, 0]
        return [response[:-2], response[-2], response[-1], 0]

    @method()
    def ResponseStatus(self, response_id: 'u', ok: 'b'):
        pass

    def close(self):
        self.child.terminate()
        try:
            self.child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            self.child.kill()
            self.child.wait()

async def run(args):
    bus = await MessageBus(bus_type=BusType.SYSTEM).connect()
    app = HostApp(str(Path(args.binary).resolve()), str(Path(args.out).resolve()))
    registered = False
    mode_id = None
    try:
        bus.export(PATH, app)
        introspection = await bus.introspect('org.sailfishos.nfc.daemon', '/')
        proxy = bus.get_proxy_object('org.sailfishos.nfc.daemon', '/', introspection)
        daemon = proxy.get_interface('org.sailfishos.nfc.Daemon')
        await daemon.call_register_local_host_app(PATH, 'Linux Tap laboratory', AID, 0)
        registered = True
        mode_id = await daemon.call_request_mode(0x08, 0)
        stop = asyncio.Event()
        for sig in (signal.SIGINT, signal.SIGTERM):
            asyncio.get_running_loop().add_signal_handler(sig, stop.set)
        print('NFCD_BRIDGE_REGISTERED transport_only=true hardware_unqualified=true', flush=True)
        await stop.wait()
    finally:
        if registered:
            await daemon.call_unregister_local_host_app(PATH)
        if mode_id is not None:
            await daemon.call_release_mode(mode_id)
        app.close()
        bus.disconnect()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True)
    parser.add_argument('--out', required=True)
    asyncio.run(run(parser.parse_args()))
