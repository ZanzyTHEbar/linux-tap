#!/usr/bin/env python3
"""Private virtual PC/SC: fixture checkout or public-invoice delivery only.

Build dependencies first with pcsc-lab-build.sh, then build the pcsc-reader feature.
This runner never downloads, installs, starts a system service or opens TCP.
"""
import argparse
import ctypes as c
import fcntl
import json
import os
from pathlib import Path
import select
import signal
import socket
import stat
import struct
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
import demo

# Direct convention, TD1 advertises T=1, no historical bytes, valid XOR TCK.
ATR = bytes.fromhex('3b800181')
READER = 'Linux Tap private VPCD 00 00'
SCRIPT = Path(__file__).resolve()


def evidence(directory, name, value):
    (directory / name).write_text(json.dumps(value, indent=2) + '\n')


def private(path):
    info = path.lstat()
    demo.require(stat.S_ISDIR(info.st_mode) and not path.is_symlink() and info.st_uid == os.getuid()
                 and stat.S_IMODE(info.st_mode) == 0o700,
                 f'Not a private owned path: {path}')


def exact(connection, length):
    data = bytearray()
    while len(data) < length:
        chunk = connection.recv(length - len(data))
        demo.require(chunk, 'Truncated VICC frame / connection closed')
        data.extend(chunk)
    return bytes(data)


def vicc(directory, delivery_only=False):
    """Only ATR/control/u16BE framing; all APDU interpretation stays in Rust."""
    private(directory)
    signal.signal(signal.SIGTERM, lambda *_: sys.exit('VICC stopped by owner'))
    with (directory / 'receiver.stderr').open('ab') as errors, \
            (directory / 'vicc.jsonl').open('a', buffering=1) as transcript, \
            socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
        receiver = demo.launch(directory, 'rust-apdu-receiver',
            [str(demo.BINARY), 'apdu-stdio', '--out', str(directory / 'invoice-inbox.json')],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors)
        try:
            server.bind(str(directory / 'vicc.sock'))
            os.chmod(directory / 'vicc.sock', 0o600)
            server.listen(1)
            server.settimeout(15)
            print('VICC_READY', flush=True)
            connection, _ = server.accept()
            with connection:
                peer = struct.unpack('3i', connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12))
                demo.require(peer[1] == os.getuid(), 'VICC peer UID mismatch')
                evidence(directory, 'vicc-peer.json', {'pid': peer[0], 'uid': peer[1], 'gid': peer[2]})
                connection.settimeout(10)
                while True:
                    if delivery_only and (directory / 'finish-receiver').exists():
                        # Parent requests EOF only after nfc-push ACK and exact inbox comparison.
                        # No APDU parsing or wallet approval policy belongs in this adapter.
                        receiver.stdin.close()
                        code = receiver.wait(timeout=5)
                        demo.require(code == 0, 'Rust receiver did not exit successfully')
                        evidence(directory, 'receiver-completed.json', {'exit_code': code,
                                 'stdin_eof': True, 'chain_submission': False})
                        return
                    if delivery_only and not select.select([connection], [], [], 0.1)[0]:
                        continue
                    size = int.from_bytes(exact(connection, 2), 'big')
                    demo.require(0 < size <= 260, f'Oversized/empty VICC frame: {size}')
                    request = exact(connection, size)
                    event = {'request': request.hex(), 'length_u16be': size}
                    response = None
                    if size == 1:
                        control = request[0]
                        demo.require(control in (0, 1, 2, 4), 'Unknown VICC control')
                        event['control'] = {0: 'OFF', 1: 'ON', 2: 'RESET', 4: 'ATR'}[control]
                        line = b'RESET\n' if control != 4 else None
                        if control == 4:
                            response = ATR
                    else:
                        line = request.hex().encode() + b'\n'
                    if line is not None:
                        receiver.stdin.write(line)
                        receiver.stdin.flush()
                        demo.require(select.select([receiver.stdout], [], [], 5)[0], 'Rust receiver deadline')
                        answer = receiver.stdout.readline(1025)
                        demo.require(answer.endswith(b'\n') and len(answer) <= 1024, 'Invalid Rust response')
                        answer = bytes.fromhex(answer.decode().strip())
                        if size == 1:
                            demo.require(answer == b'\x90\x00', 'Rust reset failed')
                            event['assembly_reset_response'] = answer.hex()
                        else:
                            response = answer
                    if response is not None:
                        demo.require(0 < len(response) <= 4096, 'Invalid response size')
                        connection.sendall(len(response).to_bytes(2, 'big') + response)
                        event['response'] = response.hex()
                    transcript.write(json.dumps(event) + '\n')
        finally:
            demo.stop(receiver)
            receiver.stdin.close()
            receiver.stdout.close()


def reset_probe(prefix, directory):
    """One native PC/SC reset assertion, not a substitute invoice implementation.

    Linux pcsc-lite wintypes.h uses long handles and unsigned long DWORDs.
    This is deliberately limited to that ABI and the locally built library.
    """
    lib = c.CDLL(str(prefix / 'lib/libpcsclite.so.1'))
    long, ulong, ptr = c.c_long, c.c_ulong, c.c_void_p
    signatures = {
        'SCardEstablishContext': [ulong, ptr, ptr, c.POINTER(long)],
        'SCardConnect': [long, c.c_char_p, ulong, ulong, c.POINTER(long), c.POINTER(ulong)],
        'SCardReconnect': [long, ulong, ulong, ulong, c.POINTER(ulong)],
        'SCardTransmit': [long, ptr, ptr, ulong, ptr, ptr, c.POINTER(ulong)],
        'SCardDisconnect': [long, ulong], 'SCardReleaseContext': [long],
    }
    for name, args in signatures.items():
        getattr(lib, name).argtypes = args
        getattr(lib, name).restype = long

    def call(name, *args):
        status = getattr(lib, name)(*args)
        demo.require(status == 0, f'{name}: {status:#x}')

    class Pci(c.Structure):
        _fields_ = [('protocol', ulong), ('length', ulong)]

    context, card, protocol = long(), long(), ulong()
    call('SCardEstablishContext', 0, None, None, c.byref(context))
    connected = False
    try:
        call('SCardConnect', context, READER.encode(), 2, 2, c.byref(card), c.byref(protocol))
        connected = True
        demo.require(protocol.value == 2, 'Expected negotiated T=1')
        pci = Pci(2, c.sizeof(Pci))

        def transmit(frame):
            answer, size = c.create_string_buffer(4096), ulong(4096)
            call('SCardTransmit', card, c.byref(pci), frame, len(frame), None, answer, c.byref(size))
            return answer.raw[:size.value]

        frames = (directory / 'expected-frames.stdout').read_bytes().splitlines()
        for frame in frames[:3]:
            demo.require(transmit(bytes.fromhex(frame.decode())).endswith(b'\x90\x00'), 'Partial assembly failed')
        # Reset through the real resource manager / IFD driver, not adapter injection.
        call('SCardReconnect', card, 2, 2, 1, c.byref(protocol))
        demo.require(transmit(bytes.fromhex(frames[-1].decode())) == b'\x6a\x80',
                     'Reset must discard partial assembly')
        evidence(directory, 'reset.json', {'native_pcsc_reconnect': True, 'protocol': 'T=1',
                 'partial_commit_after_reset': '6a80', 'atr_hex': ATR.hex()})
    finally:
        if connected:
            call('SCardDisconnect', card, 0)
        call('SCardReleaseContext', context)


def sockets(directory, process):
    """Assert every current owned daemon socket is AF_UNIX; retain inode evidence."""
    unix = Path('/proc/net/unix').read_text()
    inodes = {line.split()[6] for line in unix.splitlines()[1:]}
    owned = []
    for fd in Path(f'/proc/{process.pid}/fd').iterdir():
        try:
            target = os.readlink(fd)
        except FileNotFoundError:
            continue
        if target.startswith('socket:['):
            inode = target[8:-1]
            demo.require(inode in inodes, f'Owned daemon has a non-Unix socket: {target}')
            owned.append(target)
    demo.require(owned, 'No daemon sockets captured')
    evidence(directory, 'daemon-sockets.json', {'pid': process.pid, 'fds': owned,
             'owned_unix_rows': [line for line in unix.splitlines()[1:]
                                 if f'socket:[{line.split()[6]}]' in owned],
             'all_owned_sockets_af_unix': True})


def output_path(path):
    demo.require(path.is_absolute() and path.parent.resolve(strict=True) == path.parent,
                 'Output requires a canonical absolute private parent')
    private(path.parent)
    demo.require(not os.path.lexists(path), 'Output must be new; files and links are never replaced')


def publish(path, invoice):
    """Atomic no-replace publication of complete bytes; final file has one link."""
    output_path(path)
    fd, staging = tempfile.mkstemp(prefix='.invoice-', dir=path.parent)
    staging = Path(staging)
    try:
        with os.fdopen(fd, 'wb') as output:
            output.write(invoice)
            output.flush()
            os.fsync(output.fileno())
        # link() is atomic and refuses even dangling symlinks. Only our new staging
        # file is unlinked; this never removes or replaces an existing destination.
        os.link(staging, path, follow_symlinks=False)
    finally:
        staging.unlink()
    parent = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(parent)
    finally:
        os.close(parent)
    info = path.lstat()
    demo.require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1
                 and info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o600,
                 'Published invoice must be a private single-link regular file')


def run(prefix, automated, invoice_path=None, out=None):
    started = time.monotonic()
    if out is not None:
        output_path(out)
    demo.require(prefix == prefix.resolve(), 'Use the canonical private prefix (no symlink aliases)')
    private(prefix)
    private(prefix / 'run')
    with (prefix / 'run/lab.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        daemon_socket = prefix / 'run/pcscd.comm'
        demo.require(not daemon_socket.exists() and not (prefix / 'run/pcscd.pid').exists(),
                     'Private runtime already exists; do not delete another runtime')
        cache = prefix.parent
        subprocess.run(['sha256sum', '-c', 'build.sha256'], cwd=cache, check=True)
        demo.require((cache / 'vpcd-private.patch').read_bytes()
                     == SCRIPT.with_name('pcsc-lab-vpcd.patch').read_bytes(), 'Rebuild changed vpcd patch')
        header = (cache / 'pcsc-build/pcscd.h').read_text()
        config = (cache / 'pcsc-build/config.h').read_text()
        demo.require(f'#define PCSCLITE_IPC_DIR\t\t"{prefix}/run"' in header,
                     'Daemon was not compiled for this private runtime prefix')
        for feature in ('USE_USB', 'USE_LIBSYSTEMD', 'HAVE_LIBUDEV', 'HAVE_LIBUSB', 'HAVE_POLKIT'):
            demo.require(f'#define {feature} ' not in config, f'Unexpected host integration: {feature}')
        directory = Path(tempfile.mkdtemp(prefix='ltap-pcsc-', dir='/tmp/opencode'))
        print(f'STAGE B · virtual PC/SC only · evidence {directory}', flush=True)
        evidence(directory, 'configuration.json', {'prefix': str(prefix), 'reader': READER,
                 'daemon_socket': str(daemon_socket), 'vicc_socket': str(directory / 'vicc.sock'),
                 'pcsc_version': '2.3.3', 'atr_hex': ATR.hex(), 'usb_discovery': False,
                 'vpcd_revision': '93286a90d2411120ae17299f9902e1fdd8954185',
                 'approval_profile': 'NONE_DELIVERY_ONLY' if invoice_path else 'DevelopmentSoftware',
                 'fixture_autoapproval': automated, 'delivery_only': invoice_path is not None,
                 'systemd_activation': False, 'chain_submission': False})
        (directory / 'build.sha256').write_bytes((cache / 'build.sha256').read_bytes())
        (directory / 'pcscd.h').write_bytes((cache / 'pcsc-build/pcscd.h').read_bytes())
        (directory / 'config.h').write_bytes((cache / 'pcsc-build/config.h').read_bytes())
        os.environ['PCSCLITE_CSOCK_NAME'] = str(daemon_socket)
        os.environ['LD_LIBRARY_PATH'] = str(prefix / 'lib')
        # Refuse inherited redirection/spy settings; do not ever autostart host pcscd.
        for name in ('PCSCLITE_DELEGATE', 'PCSCLITE_LIB', 'LIBPCSCLITE_DELEGATE'):
            os.environ.pop(name, None)
        conf = directory / 'reader.conf.d'
        conf.mkdir(mode=0o700)
        (conf / 'vpcd').write_text(f'FRIENDLYNAME "Linux Tap private VPCD"\n'
            f'DEVICENAME {directory}/vicc.sock:0\nLIBPATH {prefix}/lib/libifdvpcd.so\nCHANNELID 0\n')
        processes = []
        try:
            if invoice_path is not None:
                # A bounded byte snapshot, not an invoice/crypto parser. Rust's
                # apdu-frames below enforces SignedInvoice and native MAX_ENVELOPE.
                fd = os.open(invoice_path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
                with os.fdopen(fd, 'rb') as source:
                    demo.require(stat.S_ISREG(os.fstat(source.fileno()).st_mode),
                                 'Public invoice must be a regular file')
                    invoice = source.read(8193)
                demo.require(len(invoice) <= 8192, 'Public invoice exceeds native MAX_ENVELOPE=8192')
                (directory / 'invoice.json').write_bytes(invoice)
                demo.captured(directory, 'input-frames', [str(demo.BINARY), 'apdu-frames',
                              '--input', str(directory / 'invoice.json')])
            with (directory / 'vicc.stderr').open('ab') as error, (directory / 'pcscd.log').open('ab') as log:
                adapter_command = [sys.executable, str(SCRIPT), '--vicc', str(directory)]
                if invoice_path is not None:
                    adapter_command.append('--delivery-only')
                adapter = demo.launch(directory, 'vicc', adapter_command,
                    stdout=subprocess.PIPE, stderr=error)
                processes.append(adapter)
                demo.require(select.select([adapter.stdout], [], [], 10)[0]
                             and adapter.stdout.readline().strip() == b'VICC_READY', 'VICC startup failed')
                daemon = demo.launch(directory, 'private-pcscd',
                    [str(prefix / 'sbin/pcscd'), '--foreground', '--debug', '--apdu', '--config', str(conf)],
                    stdout=log, stderr=log)
                processes.append(daemon)
                deadline = time.monotonic() + 10
                while not daemon_socket.exists() and time.monotonic() < deadline:
                    demo.require(daemon.poll() is None, 'Private pcscd exited')
                    time.sleep(0.05)
                demo.require(daemon_socket.exists(), 'Private daemon socket deadline')
                # Upstream chmods its run dir 0755/socket 0666. The enclosing
                # prefix stays 0700 throughout startup; restore strict modes
                # before clients and after failures without patching pcsc-lite.
                os.chmod(prefix / 'run', 0o700)
                os.chmod(daemon_socket, 0o600)
                if (prefix / 'run/pcscd.pid').exists():
                    os.chmod(prefix / 'run/pcscd.pid', 0o600)
                inventory = demo.captured(directory, 'readers', [str(demo.BINARY), 'readers']).decode().splitlines()
                demo.require(inventory == [READER], f'Unexpected private reader inventory: {inventory}')

                def transport(destination):
                    invoice = (destination / 'invoice.json').read_bytes()
                    inbox = destination / 'invoice-inbox.json'
                    demo.captured(destination, 'expected-frames',
                        [str(demo.BINARY), 'apdu-frames', '--input', str(destination / 'invoice.json')])
                    demo.captured(destination, 'pcsc-reset', [sys.executable, str(SCRIPT),
                        '--reset', str(destination), '--prefix', str(prefix)])
                    demo.require(not inbox.exists(), 'Partial reset probe must not publish an invoice')
                    output = demo.captured(destination, 'nfc-push', [str(demo.BINARY), 'nfc-push',
                        '--input', str(destination / 'invoice.json'), '--reader', READER])
                    demo.require(b'INVOICE_TRANSPORT_ACK_ONLY' in output and inbox.read_bytes() == invoice,
                                 'Real merchant PC/SC transfer did not persist exact invoice')
                    sockets(directory, daemon)

                def deliver(destination):
                    invoice = demo.rows(destination, 'merchant.db', 'SELECT value FROM demo_invoice')[0][0].encode()
                    inbox = destination / 'invoice-inbox.json'
                    if inbox.exists():
                        demo.require(inbox.read_bytes() == invoice, 'Invoice changed on replay')
                        return
                    (destination / 'invoice.json').write_bytes(invoice)
                    transport(destination)

                if invoice_path is not None:
                    transport(directory)
                    (directory / 'finish-receiver').touch(exist_ok=False)
                    demo.require(adapter.wait(timeout=10) == 0, 'VICC/receiver completion failed')
                    completed = json.loads((directory / 'receiver-completed.json').read_text())
                    demo.require(completed['exit_code'] == 0, 'Receiver exit not confirmed')
                else:
                    demo.checkout(directory, automated, deliver=deliver, result_name='application-result.json')
                    # Power/reset cannot erase the approved artifact or ready inbox.
                    before = (directory / 'invoice-inbox.json').read_bytes()
                    prepared = demo.rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared')
                    demo.captured(directory, 'post-checkout-reset', [sys.executable, str(SCRIPT),
                        '--reset', str(directory), '--prefix', str(prefix)])
                    demo.require((directory / 'invoice-inbox.json').read_bytes() == before
                                  and demo.rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared') == prepared,
                                  'Reset changed a ready inbox or approved artifact')
                events = [json.loads(line) for line in (directory / 'vicc.jsonl').read_text().splitlines()]
                frames = (directory / 'expected-frames.stdout').read_text().splitlines()
                apdus = [event for event in events if event['length_u16be'] > 1]
                requests = [event['request'] for event in apdus]
                demo.require(any(requests[i:i + len(frames)] == frames for i in range(len(requests))),
                              'VICC did not receive the exact merchant APDU sequence')
                demo.require(any(requests[i:i + len(frames)] == frames
                                 and all(event.get('response', '').endswith('9000')
                                         for event in apdus[i:i + len(frames)])
                                 for i in range(len(requests))), 'Missing successful APDU transcript responses')
                demo.require(any(event.get('control') == 'RESET' for event in events), 'No native reset evidence')
                demo.require((adapter.poll() == 0 if invoice_path else adapter.poll() is None)
                             and daemon.poll() is None, 'Lab runtime failed')
        except BaseException as error:
            evidence(directory, 'failure.json', {'result': 'STOPPED', 'error': str(error), 'chain_submission': False})
            raise
        finally:
            for process in reversed(processes):
                demo.stop(process)
                if process.stdout:
                    process.stdout.close()
            os.chmod(prefix / 'run', 0o700)
            ledger = directory / 'processes.jsonl'
            identities = [json.loads(line) for line in ledger.read_text().splitlines()] if ledger.exists() else []
            remaining = []
            for identity in identities:
                path = Path(f'/proc/{identity["pid"]}/stat')
                if path.exists() and path.read_text().rsplit(')', 1)[1].split()[19] == identity['start_ticks']:
                    remaining.append(identity)
            evidence(directory, 'cleanup.json', {'remaining_owned_processes': remaining,
                     'private_daemon_socket_removed': not daemon_socket.exists(),
                     'runtime_directory_mode': oct(stat.S_IMODE((prefix / 'run').stat().st_mode))})
            demo.require(not remaining and not daemon_socket.exists(), 'Owned runtime cleanup incomplete')
        if invoice_path is not None:
            try:
                demo.require((directory / 'invoice-inbox.json').read_bytes() == invoice,
                             'Completed receiver inbox differs from source snapshot')
                publish(out, invoice)
            except BaseException as error:
                evidence(directory, 'failure.json', {'result': 'STOPPED', 'error': str(error), 'chain_submission': False})
                raise
            result = {'result': 'APDU_DELIVERED_NOT_APPROVED_NOT_PAID', 'chain_submission': False,
                      'invoice_source': str(invoice_path), 'invoice_out': str(out),
                      'exact_invoice_bytes': True, 'receiver_exit_code': 0,
                      'wallet_approval': False, 'payment': False}
        else:
            result = json.loads((directory / 'application-result.json').read_text())
        result.update(transport='real PC/SC → Unix-only vpcd → thin VICC → Rust Receiver',
                      virtual_pcsc=True, protocol='T=1', atr_hex=ATR.hex(),
                      exact_apdu_sequence=True, reset_discards_only_assembly=True,
                      all_owned_daemon_sockets_af_unix=True, owned_runtimes_remaining=0,
                      elapsed_seconds=round(time.monotonic() - started, 3),
                      fixture_autoapproval=automated)
        evidence(directory, 'result.json', result)
        status = 'APDU_DELIVERED_NOT_APPROVED_NOT_PAID' if invoice_path else 'STAGE B PASS'
        print(f'{status} · no RF / live settlement · {directory}/result.json', flush=True)


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prefix', type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--approve-simulation', action='store_true')
    mode.add_argument('--invoice', type=Path, help='Public SignedInvoice file; transport ONLY, no signing/RPC')
    parser.add_argument('--out', type=Path, help='New inbox file in an owned canonical mode-0700 parent')
    parser.add_argument('--vicc', type=Path, help=argparse.SUPPRESS)
    parser.add_argument('--reset', type=Path, help=argparse.SUPPRESS)
    parser.add_argument('--delivery-only', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.vicc:
        vicc(args.vicc, args.delivery_only)
    elif args.reset:
        reset_probe(args.prefix, args.reset)
    else:
        if not args.prefix or not args.prefix.is_absolute():
            parser.error('--prefix must name the privately built absolute prefix')
        if (args.invoice is None) != (args.out is None):
            parser.error('--invoice and --out must be supplied together')
        if args.delivery_only:
            parser.error('--delivery-only is internal to the VICC adapter')
        if not args.invoice and not args.approve_simulation and not os.isatty(0):
            parser.error('Use a terminal or explicitly --approve-simulation (public fixtures only)')
        run(args.prefix, args.approve_simulation, args.invoice, args.out)


if __name__ == '__main__':
    main()
