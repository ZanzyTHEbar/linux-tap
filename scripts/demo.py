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
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/debug/linux-tap'


def stage(title):
    print(f'\n── {title} ──', flush=True)


def cycle(directory, automated=False, interrupt=False):
    socket = directory / 'checkout.sock'
    with (directory / 'merchant.log').open('a') as log:
        merchant = subprocess.Popen(
            [str(BINARY), 'serve-demo', '--socket', str(socket),
             '--db', str(directory / 'merchant.db')],
            stdout=subprocess.PIPE, stderr=log, text=True)
        try:
            ready, _, _ = select.select([merchant.stdout], [], [], 10)
            if not ready or not merchant.stdout.readline().startswith('SIMULATION_READY'):
                raise RuntimeError('Merchant did not become ready; inspect merchant.log')
            command = [str(BINARY), 'pay-demo', '--socket', str(socket),
                       '--journal', str(directory / 'wallet.db'),
                       '--approve-simulation' if automated else '--interactive-simulation']
            if interrupt:
                command.append('--disconnect-after-journal')
            # Inherit the terminal so exact terms and approval come from Rust, not this script.
            subprocess.run(command, check=True, timeout=130)
            if merchant.wait(timeout=25) != 0:
                raise RuntimeError('Merchant failed; inspect merchant.log')
        finally:
            if merchant.poll() is None:
                merchant.terminate()
                try:
                    merchant.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    merchant.kill()
                    merchant.wait()
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
    directory = Path(tempfile.mkdtemp(prefix='ltap-'))
    print('LINUX TAP · native Linux wallet + merchant', flush=True)
    print('SIMULATION ONLY · public fixture keys · no RPC · no NFC · no funds move', flush=True)
    print(f'Evidence retained at: {directory}', flush=True)

    stage('1 / Review → approve → interrupt after durable signing, before send')
    cycle(directory, args.approve_simulation, interrupt=True)
    prepared = rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared')
    require(len(prepared) == 1, 'Expected one journaled transaction')
    require(not rows(directory, 'merchant.db', 'SELECT digest FROM simulated_payments'),
            'Payment must not be recorded before sending')
    print(f'Journal saved: {prepared[0][0]}', flush=True)
    print('Merchant records: 0. Wallet process exited. Nothing sent.', flush=True)
    if not args.approve_simulation:
        input('\nPress Enter to restart and resend the already approved bytes. ')

    stage('2 / Restart merchant + wallet → resume exact signed bytes')
    cycle(directory, automated=True)
    require(rows(directory, 'merchant.db', 'SELECT digest, value FROM simulated_payments')
            == prepared, 'Merchant must record the exact journaled transaction')
    if not args.approve_simulation:
        input('\nPress Enter to retry once more and check replay protection. ')

    stage('3 / Restart again → replay, not a second simulated payment')
    cycle(directory, automated=True)
    require(rows(directory, 'wallet.db', 'SELECT digest, value FROM prepared') == prepared,
            'Retry must not replace signed bytes')
    require(rows(directory, 'merchant.db', 'SELECT digest, value FROM simulated_payments')
            == prepared, 'Replay must leave exactly one simulated payment')
    require(rows(directory, 'wallet.db', 'SELECT digest FROM simulated_receipts')
            == [(prepared[0][0],)], 'Wallet must retain the matching simulated receipt')

    stage('4 / Experimental APDU transport reconstruction — software only')
    invoice = rows(directory, 'merchant.db', 'SELECT value FROM demo_invoice')[0][0]
    invoice_path = directory / 'invoice.json'
    invoice_path.write_text(invoice)
    invoice_path.chmod(0o600)
    subprocess.run([str(BINARY), 'apdu-roundtrip', '--input', str(invoice_path)],
                   check=True, timeout=25)
    result = {'result': 'PASS', 'transport': 'private Unix socket',
              'prepared_transactions': 1, 'simulated_payments': 1,
              'exact_signed_bytes_preserved': True, 'chain_submission': False,
              'live_settlement_verified': False, 'physical_nfc_test': False}
    with (directory / 'result.json').open('x') as output:
        json.dump(result, output, indent=2)
    stage('PASS / 1 signed transaction · 1 simulated payment · identical replay')
    print('No live settlement, gasless eligibility or hardware compatibility demonstrated.', flush=True)
    print(f'Evidence: {directory}/result.json and merchant.log', flush=True)


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        raise SystemExit(f'DEMO STOPPED: {error}. No PASS result; inspect the retained evidence.')
    except (KeyboardInterrupt, EOFError):
        raise SystemExit('DEMO STOPPED: interrupted or input closed. State retained; no automatic retry.')
