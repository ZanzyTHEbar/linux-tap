#!/usr/bin/env python3
"""Exercise real CLI processes, durable retry and APDU reconstruction without a chain."""
import json
import os
import sqlite3
import subprocess
import tempfile
import time
from contextlib import closing
from pathlib import Path
import demo

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/debug/linux-tap'

def run(*args):
    return subprocess.run([str(BINARY), *map(str, args)], check=True,
                          capture_output=True, text=True, timeout=25)

def cycle(database, wallet, interrupt=False):
    merchant = demo.launch(database.parent, 'smoke-merchant',
                           [str(BINARY), 'serve-demo', '--stdio', '--db', str(database)],
                           stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    customer = None
    try:
        command=[str(BINARY), 'pay-demo', '--stdio', '--journal', str(wallet), '--approve-simulation']
        if interrupt:
            command.append('--disconnect-after-journal')
        customer=demo.launch(database.parent, 'smoke-wallet', command,
                             stdin=merchant.stdout, stdout=merchant.stdin, stderr=subprocess.PIPE)
        _, output=customer.communicate(timeout=25)
        assert customer.returncode==0,output.decode()
        if interrupt:
            merchant.terminate()
            merchant.communicate(timeout=5)
        else:
            _, error=merchant.communicate(timeout=10)
            assert merchant.returncode==0,error.decode()
        return output.decode()
    finally:
        for process in (customer, merchant):
            if process is not None and process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)

def saved_artifact(database):
    assert not Path(f'{database}-wal').exists(), 'Native close must finish its WAL checkpoint'
    with closing(sqlite3.connect(f'file:{database}?mode=ro', uri=True)) as connection:
        assert connection.execute('PRAGMA journal_mode').fetchone()[0] == 'wal'
        return connection.execute('SELECT invoice_hash,value,digest FROM prepared').fetchall()

def main():
    assert BINARY.is_file(), 'Run cargo build --locked -p tap-cli first'
    os.umask(0o077)
    storage = json.loads(run('storage-info').stdout)
    assert tuple(map(int, storage['sqlite_version'].split('.'))) >= (3, 51, 3), storage
    with tempfile.TemporaryDirectory(prefix='linux-tap-') as temporary:
        directory = Path(temporary)
        merchant_db=directory/'merchant.db'
        wallet_db=directory/'wallet.db'
        assert 'SIMULATED_INTERRUPT_BEFORE_SEND' in cycle(merchant_db,wallet_db,True)
        # Native process exit checkpoints/closes its real WAL; reopen must retain exact bytes.
        original = saved_artifact(wallet_db)
        assert len(original) == 1
        assert '\"replay\": false' in cycle(merchant_db,wallet_db)
        assert '\"replay\": true' in cycle(merchant_db,wallet_db)
        assert saved_artifact(wallet_db) == original
        with sqlite3.connect(wallet_db) as connection:
            assert connection.execute('SELECT COUNT(*) FROM prepared').fetchone()[0] == 1
            wallet_digest = connection.execute('SELECT digest FROM prepared').fetchone()[0]
        with sqlite3.connect(merchant_db) as connection:
            assert connection.execute('SELECT COUNT(*) FROM simulated_payments').fetchone()[0] == 1
            assert connection.execute('SELECT digest FROM simulated_payments').fetchone()[0] == wallet_digest
            assert connection.execute('SELECT value FROM simulated_payments').fetchone()[0] == original[0][1]
            invoice = connection.execute('SELECT value FROM demo_invoice').fetchone()[0]
        invoice_file = directory / 'invoice.json'
        invoice_file.write_text(invoice)
        assert 'APDU_SIMULATOR_OK' in run('apdu-roundtrip', '--input', invoice_file).stdout
        assert (os.stat(wallet_db).st_mode & 0o077) == 0
        print(json.dumps({'result': 'PASS', 'processes': 'native Rust wallet and merchant over pipes',
                          'interruption_recovered': True, 'merchant_restart_recovered': True,
                          'sqlite_runtime': storage, 'exact_artifact_after_checkpoint_reopen': True,
                          'prepared_transactions': 1, 'simulated_payments': 1,
                          'chain_submission': False, 'physical_nfc_test': False}, indent=2))

if __name__ == '__main__':
    main()
