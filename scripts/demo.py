#!/usr/bin/env python3
"""Recording-friendly checkout using real native processes and private Unix sockets.

Python only directs the demo and reads evidence; Rust owns validation and signing.
All state is retained in a newly created private directory. Never broadcasts.
"""
import argparse
import json
import os
from pathlib import Path
import select
import sqlite3
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/debug/linux-tap'


def stage(title):
    print(f'\n── {title} ──', flush=True)


def launch(directory, role, command, **kwargs):
    process = subprocess.Popen(command, **kwargs)
    # Linux start identity is evidence only, never an authority to kill a PID later.
    try:
        stat = Path(f'/proc/{process.pid}/stat')
        identity = stat.read_text().rsplit(')', 1)[1].split()[19] if stat.exists() else None
        with (directory / 'processes.jsonl').open('a') as log:
            log.write(json.dumps({'role': role, 'pid': process.pid, 'start_ticks': identity}) + '\n')
    except BaseException:
        stop(process)
        for stream in (process.stdin, process.stdout, process.stderr):
            if stream:
                stream.close()
        raise
    return process


def stop(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def captured(directory, role, command, data=None):
    process = launch(directory, role, command, stdin=subprocess.PIPE,
                     stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        output, error = process.communicate(data, timeout=25)
        (directory / f'{role}.stdout').write_bytes(output)
        (directory / f'{role}.stderr').write_bytes(error)
        require(process.returncode == 0, f'{role} failed: {error.decode()}')
        return output
    finally:
        stop(process)


def deliver_invoice(directory):
    invoice = rows(directory, 'merchant.db', 'SELECT value FROM demo_invoice')[0][0].encode()
    source = directory / 'invoice.json'
    inbox = directory / 'invoice-inbox.json'
    if inbox.exists():
        require(inbox.read_bytes() == invoice, 'Merchant changed the APDU-delivered invoice')
        return
    with source.open('xb') as output:
        output.write(invoice)
    frames = captured(directory, 'apdu-frames',
                      [str(BINARY), 'apdu-frames', '--input', str(source)])
    responses = captured(directory, 'apdu-receiver',
                         [str(BINARY), 'apdu-stdio', '--out', str(inbox)], frames)
    require(len(responses.splitlines()) == len(frames.splitlines())
            and all(line.endswith(b'9000') for line in responses.splitlines()),
            'Receiver did not ACK every frame')
    require(inbox.read_bytes() == invoice, 'Receiver must persist the exact invoice')
    print('APDU transport ACK only. Complete inbox ready; not approved or paid.', flush=True)


def cycle(directory, automated=False, interrupt=False, lose_receipt=False, replay_only=False,
          deliver=deliver_invoice):
    socket = directory / 'checkout.sock'
    with (directory / 'merchant.log').open('a') as log:
        command = [str(BINARY), 'serve-demo', '--socket', str(socket),
                   '--db', str(directory / 'merchant.db')]
        if lose_receipt:
            command.append('--drop-receipt-after-commit')
        merchant = launch(directory, 'merchant', command,
            stdout=subprocess.PIPE, stderr=log, text=True)
        try:
            ready, _, _ = select.select([merchant.stdout], [], [], 10)
            if not ready or not merchant.stdout.readline().startswith('SIMULATION_READY'):
                raise RuntimeError('Merchant did not become ready; inspect merchant.log')
            deliver(directory)
            command = [str(BINARY), 'pay-demo', '--socket', str(socket),
                       '--journal', str(directory / 'wallet.db'),
                       '--invoice-inbox', str(directory / 'invoice-inbox.json'),
                       '--approve-simulation' if automated else '--interactive-simulation']
            if interrupt:
                command.append('--disconnect-after-journal')
            if replay_only:
                command.append('--replay-only')
            # Input is the real terminal. Tee Rust's actual prompt/output into retained evidence.
            wallet = launch(directory, 'wallet', command, stderr=subprocess.PIPE)
            output = bytearray()
            try:
                deadline = time.monotonic() + 130
                with (directory / 'wallet.log').open('ab') as wallet_log:
                    while True:
                        remaining = deadline - time.monotonic()
                        ready, _, _ = select.select([wallet.stderr], [], [], max(0, remaining))
                        if not ready:
                            raise RuntimeError('Wallet deadline exceeded')
                        chunk = os.read(wallet.stderr.fileno(), 65536)
                        if not chunk:
                            break
                        output.extend(chunk)
                        wallet_log.write(chunk)
                        wallet_log.flush()
                        sys.stderr.buffer.write(chunk)
                        sys.stderr.buffer.flush()
                    code = wallet.wait(timeout=max(0, deadline - time.monotonic()))
            finally:
                stop(wallet)
                wallet.stderr.close()
            output = output.decode()
            require('APDU_INBOX_CONSUMED' in output, 'Wallet did not consume the APDU inbox')
            if lose_receipt:
                require(code != 0 and 'SIMULATED_OUTCOME_UNKNOWN artifact_retained=true' in output,
                        'Lost receipt must report unresolved outcome, not success')
            else:
                require(code == 0, 'Wallet failed; inspect wallet.log')
            if replay_only:
                require('RESUMING_SAVED_ARTIFACT no_new_signing=true' in output
                        and 'SIMULATION_APPROVAL' not in output,
                        'Restart must load the saved artifact, not approve a new payment')
            if merchant.wait(timeout=25) != 0:
                raise RuntimeError('Merchant failed; inspect merchant.log')
            return output
        finally:
            stop(merchant)
            merchant.stdout.close()


def rows(directory, database, query):
    with sqlite3.connect(f'file:{directory / database}?mode=ro', uri=True) as connection:
        return connection.execute(query).fetchall()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--approve-simulation', action='store_true',
                        help='Explicit public-fixture approval for unattended software checks')
    args = parser.parse_args()
    if not BINARY.is_file():
        parser.error('Run cargo build --locked -p tap-cli first (default target directory)')
    if not args.approve_simulation and not os.isatty(0):
        parser.error('Use a terminal for approval, or --approve-simulation for automation')
    # A short path avoids the Linux Unix-socket path length limit. mkdtemp creates mode 0700.
    os.umask(0o077)
    directory = Path(tempfile.mkdtemp(prefix='ltap-'))
    print('LINUX TAP · native Linux wallet + merchant', flush=True)
    print('SIMULATION ONLY · public fixture keys · no RPC · no RF · no funds move', flush=True)
    print(f'Evidence retained at: {directory}', flush=True)

    try:
        checkout(directory, args.approve_simulation)
    except BaseException as error:
        with (directory / 'failure.json').open('x') as output:
            json.dump({'result': 'STOPPED', 'error': str(error), 'chain_submission': False}, output, indent=2)
        raise


def checkout(directory, automated, deliver=deliver_invoice, result_name='result.json'):
    stage('1 / APDU inbox → review → approve → interrupt before send')
    cycle(directory, automated, interrupt=True, deliver=deliver)
    prepared = rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared')
    require(len(prepared) == 1, 'Expected one journaled transaction')
    require(not rows(directory, 'merchant.db', 'SELECT digest FROM simulated_payments'),
            'Payment must not be recorded before sending')
    print(f'Journal saved: {prepared[0][0]}', flush=True)
    print('Merchant records: 0. Wallet process exited. Nothing sent.', flush=True)
    if not automated:
        input('\nPress Enter to send the saved bytes and deliberately lose the merchant receipt. ')

    stage('2 / Merchant commits → receipt lost → wallet outcome UNKNOWN')
    cycle(directory, automated=True, lose_receipt=True, replay_only=True, deliver=deliver)
    require(rows(directory, 'merchant.db', 'SELECT digest, value FROM simulated_payments')
            == prepared, 'Merchant must record the exact journaled transaction')
    require(not rows(directory, 'wallet.db', 'SELECT digest FROM simulated_receipts'),
            'Wallet must not invent a receipt after the reply was lost')
    require(rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared') == prepared,
            'Lost reply must retain exact BCS, signature and digest')
    require('SIMULATED_RECEIPT_DROPPED_AFTER_COMMIT' in (directory / 'merchant.log').read_text(),
            'Merchant fault must happen after commit')
    if not automated:
        input('\nPress Enter to restart both processes and replay only the saved artifact. ')

    stage('3 / Restart both → same artifact → existing merchant receipt')
    require('"replay": true' in cycle(directory, automated=True, replay_only=True, deliver=deliver),
            'Merchant must identify the already committed payment as a replay')
    stage('4 / Duplicate replay → still one simulated payment')
    require('"replay": true' in cycle(directory, automated=True, replay_only=True, deliver=deliver),
            'Duplicate must return replay')
    require(rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared') == prepared,
            'Retry must not replace signed bytes')
    require(rows(directory, 'merchant.db', 'SELECT digest, value FROM simulated_payments')
            == prepared, 'Replay must leave exactly one simulated payment')
    require(rows(directory, 'wallet.db', 'SELECT digest FROM simulated_receipts')
            == [(prepared[0][0],)], 'Wallet must retain the matching simulated receipt')

    result = {'result': 'PASS', 'transport': 'emulated APDU ingress; private Unix socket claim/receipt',
              'apdu_invoice_consumed_by_wallet': True,
              'receipt_lost_after_merchant_commit': True,
              'unknown_outcome_without_wallet_receipt': True,
              'replay_only_no_new_signing': True,
              'artifact_fields_compared': ['bcs_hex', 'signature', 'digest'],
              'prepared_transactions': 1, 'simulated_payments': 1, 'wallet_receipt_associations': 1,
              'exact_signed_bytes_preserved': True, 'chain_submission': False,
              'live_settlement_verified': False, 'physical_nfc_test': False}
    with (directory / result_name).open('x') as output:
        json.dump(result, output, indent=2)
    stage('PASS / 1 signed transaction · 1 simulated payment · identical replay')
    print('No live settlement, gasless eligibility or hardware compatibility demonstrated.', flush=True)
    print(f'Evidence: {directory}/{result_name} and merchant.log', flush=True)


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        raise SystemExit(f'DEMO STOPPED: {error}. No PASS result; inspect the retained evidence.')
    except (KeyboardInterrupt, EOFError):
        raise SystemExit('DEMO STOPPED: interrupted or input closed. State retained; no automatic retry.')
