#!/usr/bin/env python3
"""OPT-IN human testnet fault trial; keyless, no signing/preparation or automatic resubmission.

Review required before live use. Submit inherits the real terminal and requires TESTNET SUBMIT.
Exit 86 loses an application-received success before persistence, NOT an RPC packet.
"""
import argparse
from collections import Counter
from contextlib import contextmanager
import base64
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import signal
import sqlite3
import stat
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
import demo

CHAIN = '69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD'
ASSET = '0xa1ec7fc00a6f40db9693ad1415d0c193ad3906494428cf252621037bd7117e29::usdc::USDC'
PROFILE = 'circle-sui-testnet/software-ed25519-host-approval/v1'
TABLES = ('demo_invoice prepared simulated_payments simulated_receipts '
          'testnet_candidates_v1 testnet_reservations_v1 testnet_artifacts_v1 '
          'testnet_recovery_approvals_v1 testnet_dispatch_v1 testnet_attempts_v1 '
          'testnet_wallet_observations_v1 testnet_merchant_config_v1 testnet_registrations_v1 '
          'testnet_merchant_observations_v1 testnet_credits_v1 testnet_confirmations_v1 '
          'conversion_records_v1 conversion_history_v1').split()
FILES = ('invoice.json', 'inbox.json', 'prepared.json', 'registration.json')


class TrialStopped(BaseException):
    def __init__(self, signum):
        self.signum = signum
        super().__init__(f'terminated by {signal.Signals(signum).name}; outcome may be unknown')


class Termination:
    """First catchable signal unwinds; launch/cleanup defer it and repeats cannot interrupt reaping."""
    def __init__(self):
        self.pending = None
        self.depth = 0
        self.raised = False

    def request(self, signum, _frame):
        if self.pending is None:
            self.pending = signum
        self.check()

    def check(self):
        if self.pending is not None and not self.depth and not self.raised:
            self.raised = True
            raise TrialStopped(self.pending)

    @contextmanager
    def defer(self):
        self.depth += 1
        try:
            yield
        finally:
            self.depth -= 1

    @contextmanager
    def installed(self):
        self.__init__()
        previous = {s: signal.getsignal(s) for s in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT)}
        try:
            for s in previous:
                signal.signal(s, self.request)
            yield
        finally:
            for s, handler in previous.items():
                signal.signal(s, handler)


termination = Termination()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def private(path, directory=False):
    require(path.is_absolute() and path.resolve(strict=True) == path, 'Use canonical absolute paths')
    m = path.lstat()
    require(m.st_uid == os.getuid() and stat.S_IMODE(m.st_mode) == (0o700 if directory else 0o600)
            and (stat.S_ISDIR(m.st_mode) if directory else stat.S_ISREG(m.st_mode) and m.st_nlink == 1),
            f'Expected owned private {"directory" if directory else "single-link file"}: {path}')


def read(path):
    private(path)
    with path.open('rb') as source:
        data = source.read(1_048_577)
    require(len(data) <= 1_048_576, 'Public artifact exceeds bound')
    return data


def save(directory, name, value):
    with (directory / name).open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def snapshot(path, invoice_hash):
    """Read-only WAL-aware row facts/fingerprints, never a database copy or backup."""
    private(path)
    facts = {'counts': {}, 'row_hashes': {}, 'records': {}}
    with path.open('rb') as owner:
        fcntl.flock(owner, fcntl.LOCK_EX | fcntl.LOCK_NB)
        db = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True, timeout=5)
        try:
            db.row_factory = sqlite3.Row
            db.execute('PRAGMA query_only=ON')
            db.execute('BEGIN')
            names = {r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")}
            require(names == set(TABLES), 'Unexpected journal schema; no trial started')
            size = 0
            for table in TABLES:
                hashes, records = [], []
                for row in db.execute(f'SELECT * FROM "{table}" ORDER BY rowid'):
                    row = dict(row)
                    encoded = json.dumps(row, sort_keys=True).encode()
                    size += len(encoded)
                    require(size <= 32_000_000 and len(hashes) < 10000, 'Snapshot exceeds bounded qualification size')
                    hashes.append(sha(encoded))
                    if row.get('invoice_hash') == invoice_hash or table == 'testnet_merchant_config_v1':
                        require(len(records) < 100, 'Too many trial records')
                        records.append(row)
                facts['counts'][table] = len(hashes)
                facts['row_hashes'][table] = hashes
                facts['records'][table] = records
            # An explicit operator-chosen pre-existing credit identifies the authoritative ledger.
            facts['credit_digests'] = [r[0] for r in db.execute('SELECT digest FROM testnet_credits_v1')]
        finally:
            db.close()
    return facts


def one(facts, table):
    records = facts['records'][table]
    require(len(records) == 1, f'Expected exactly one trial record: {table}')
    return json.loads(records[0]['value'])


def unchanged_rows(before, after, additions):
    for table in TABLES:
        require(not (Counter(before['row_hashes'][table]) - Counter(after['row_hashes'][table])),
                f'Existing row changed/deleted: {table}')
        delta = additions.get(table, 0)
        require(after['counts'][table] == before['counts'][table] + delta, f'Unexpected total change: {table}')
        if delta:
            require(len(after['records'][table]) == len(before['records'][table]) + delta,
                    f'New rows are not exclusively this invoice: {table}')


def run(directory, role, args, expected=0, terminal=False):
    """No stdin injection: only the submit child inherits the human's terminal."""
    process = None
    output = {'stdout': bytearray(), 'stderr': bytearray()}
    started = time.monotonic()
    try:
        # Python-level deferral, not a blocked OS signal mask inherited by the child.
        # The handle and ledger must exist before a pending signal can unwind.
        with termination.defer():
            process = demo.launch(directory, role, [str(demo.BINARY), 'testnet', *args],
                                  stdin=None if terminal else subprocess.DEVNULL,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        termination.check()
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ, 'stdout')
            selector.register(process.stderr, selectors.EVENT_READ, 'stderr')
            while selector.get_map():
                require(time.monotonic() - started < 300, 'Native stage deadline; outcome may be unknown, retain all state')
                for key, _ in selector.select(0.1):
                    chunk = os.read(key.fileobj.fileno(), 65536)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    output[key.data].extend(chunk)
                    require(sum(map(len, output.values())) <= 8_000_000, 'Native output exceeds bound')
                    stream = sys.stdout.buffer if key.data == 'stdout' else sys.stderr.buffer
                    stream.write(chunk)
                    stream.flush()
            process.wait(timeout=5)
    finally:
        with termination.defer():
            if process is not None:
                try:
                    if process.poll() is None:
                        process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                finally:
                    for name, data in output.items():
                        (directory / f'{role}.{name}').write_bytes(data)
                    save(directory, f'{role}.capture.json', {'stage': role, 'command': [str(demo.BINARY), 'testnet', *args],
                         'timeout_seconds': 300, 'real_terminal_input': terminal, 'exit_code': process.returncode,
                         'signal': termination.pending, 'waited': process.poll() is not None,
                         'elapsed_seconds': round(time.monotonic() - started, 3)})
                    process.stdout.close()
                    process.stderr.close()
        termination.check()
    require(process.returncode == expected, f'{role}: expected {expected}, got {process.returncode}; STOP, do not retry the fault or submit again')
    return bytes(output['stdout'] + output['stderr']).decode()


def execute(args, directory):
    trial = args.trial_directory
    private(trial, True)
    private(args.merchant_ledger.parent, True)
    private(args.merchant_ledger)
    require(args.merchant_ledger.parent != trial, 'Reuse the authoritative previous merchant ledger, never a new trial ledger')
    original = {name: read(trial / name) for name in FILES}
    require(original['invoice.json'] == original['inbox.json'], 'Invoice/inbox differ')
    invoice = json.loads(original['invoice.json'])
    prepared = json.loads(original['prepared.json'])
    registration = json.loads(original['registration.json'])
    invoice_hash = registration['invoice_hash']
    digest = prepared['digest']
    require(invoice['merchant_key_hex'] == args.merchant_key, 'Merchant pin mismatch')
    wallet_path = trial / 'wallet.sqlite'
    before = {'wallet': snapshot(wallet_path, invoice_hash), 'merchant': snapshot(args.merchant_ledger, invoice_hash)}
    candidate = one(before['wallet'], 'testnet_candidates_v1')
    require(candidate['profile'] == PROFILE and candidate['offline_fixture_epoch'] is None,
            'OFFLINE_FIXTURE_NEVER_SUBMITTABLE / wrong preparation profile')
    require(candidate['core_hash'] == invoice_hash and candidate['digest'] == digest
            and candidate['bcs_hex'] == prepared['bcs_hex']
            and bytes.fromhex(candidate['envelope_hex']) == original['invoice.json']
            and candidate['endpoint'] == args.endpoint and candidate['merchant_pin'] == args.merchant_key,
            'Saved artifact/candidate/profile binding mismatch')
    require(one(before['wallet'], 'testnet_artifacts_v1') == prepared, 'Exported artifact differs from journal')
    one(before['wallet'], 'testnet_reservations_v1')
    for table in ['testnet_candidates_v1', 'testnet_reservations_v1', 'testnet_artifacts_v1']:
        require(before['wallet']['counts'][table] == 1, 'Fresh trial needs a single prepared candidate')
    for table in ['testnet_dispatch_v1', 'testnet_attempts_v1', 'testnet_confirmations_v1']:
        require(before['wallet']['counts'][table] == 0, 'Existing intent/attempt/association; do not rearm')
    for record in before['wallet']['records']['testnet_wallet_observations_v1']:
        value = json.loads(record['value'])
        require(value['source'] == 'Query' and value['observation']['outcome'] == 'NotFound', 'Historical non-NotFound result; no fresh trial')
    require(args.preserve_credit_digest != digest and args.preserve_credit_digest in before['merchant']['credit_digests'],
            'Authoritative ledger must retain the explicitly chosen prior payment credit')
    config = {'profile': 'circle-sui-testnet/controlled-dispatch-host-approval/v1',
              'chain': CHAIN, 'merchant_pin': args.merchant_key, 'endpoint': args.merchant_endpoint}
    require(one(before['merchant'], 'testnet_merchant_config_v1') == config == registration['config'], 'Merchant namespace mismatch')
    require(one(before['merchant'], 'testnet_registrations_v1') == registration, 'Register fresh trial receiver in authoritative ledger first')
    require(bytes.fromhex(registration['envelope_hex']) == original['invoice.json'], 'Registration envelope differs')
    for table in ['testnet_credits_v1', 'testnet_merchant_observations_v1']:
        require(not before['merchant']['records'][table], 'Invoice already observed/credited; never rearm')
    for facts in before.values():
        require(facts['counts']['simulated_payments'] == facts['counts']['simulated_receipts'] == 0, 'Simulated state is not a live trial')
    transport = {'performed_by_runner': False, 'bound_evidence': False}
    if args.pcsc_result:
        private(args.pcsc_result.parent, True)
        result = json.loads(read(args.pcsc_result))
        require(result['result'] == 'APDU_DELIVERED_NOT_APPROVED_NOT_PAID' and result['exact_apdu_sequence']
                and result['receiver_exit_code'] == 0 and not result['fixture_autoapproval']
                and result['virtual_pcsc'] and result['all_owned_daemon_sockets_af_unix']
                and result['owned_runtimes_remaining'] == 0 and not result['chain_submission']
                and not result['wallet_approval'] and not result['payment']
                and result['protocol'] == 'T=1' and result['atr_hex'] == '3b800181'
                and result['invoice_out'] == str(trial / 'inbox.json')
                and result['invoice_source'] == str(trial / 'invoice.json'), 'PCSC evidence is not this fresh trial')
        for name in ['invoice.json', 'invoice-inbox.json']:
            require(read(args.pcsc_result.parent / name) == original['invoice.json'], 'PCSC snapshot differs')
        transport.update(bound_evidence=True, protocol=result['protocol'], atr_hex=result['atr_hex'],
                         invoice_sha256=sha(original['invoice.json']))
    save(directory, 'before.json', before)
    save(directory, 'transport.json', transport)
    require(sys.stdin.isatty(), 'Live trial requires a REAL terminal; no unattended approval mode')
    print('STARTED: HUMAN TESTNET 0.01 USDC TRIAL; native signature/template/amount guards run BEFORE approval. No automatic resubmit.', flush=True)
    wallet = ['--invoice-inbox', str(trial / 'inbox.json'), '--merchant-key', args.merchant_key,
              '--expected-sender', args.expected_sender, '--endpoint', args.endpoint,
              '--merchant-endpoint', args.merchant_endpoint, '--journal', str(wallet_path)]
    merchant = ['--invoice', str(trial / 'invoice.json'), '--merchant-key', args.merchant_key,
                '--merchant-endpoint', args.merchant_endpoint, '--ledger', str(args.merchant_ledger),
                '--prepared', str(trial / 'prepared.json')]
    receipt, replayed = directory / 'receipt.json', directory / 'replayed-receipt.json'
    require(not os.path.lexists(receipt) and not os.path.lexists(replayed), 'Receipt output collision')

    def state(name, wallet_additions, merchant_additions):
        require(all(read(trial / k) == v for k, v in original.items()), 'Original invoice/registration/inbox/full signed artifact changed')
        now = {'wallet': snapshot(wallet_path, invoice_hash), 'merchant': snapshot(args.merchant_ledger, invoice_hash)}
        unchanged_rows(before['wallet'], now['wallet'], wallet_additions)
        unchanged_rows(before['merchant'], now['merchant'], merchant_additions)
        save(directory, name + '.json', now)
        return now

    out = run(directory, 'submit-fault', ['submit', *wallet, '--registration', str(trial / 'registration.json'),
              '--fault-exit-after-execution-before-observation'], expected=86, terminal=True)
    match = re.search(r'FAULT_EXIT_AFTER_EXECUTION_BEFORE_OBSERVATION digest=' + re.escape(digest)
                      + r' invoice_hash=' + re.escape(invoice_hash) + r' attempt=(\d+) checkpoint=(\d+)', out)
    require(match is not None, 'Expected application-result loss marker not reached')
    attempt, checkpoint = map(int, match.groups())
    wc = {'testnet_dispatch_v1': 1, 'testnet_attempts_v1': 1, 'testnet_wallet_observations_v1': 1}
    lost = state('after-submit-fault', wc, {})
    require(lost['wallet']['records']['testnet_attempts_v1'][0]['id'] == attempt, 'Fault marker attempt mismatch')
    for row in lost['wallet']['records']['testnet_wallet_observations_v1']:
        v = json.loads(row['value'])
        require(v['source'] == 'Query' and v['observation']['outcome'] == 'NotFound', 'Dispatch result was persisted; fault boundary not proven')
    run(directory, 'reconcile', ['reconcile', *wallet])
    wc['testnet_wallet_observations_v1'] += 1
    recovered = state('after-reconcile', wc, {})
    observation = json.loads(recovered['wallet']['records']['testnet_wallet_observations_v1'][-1]['value'])
    require(observation['source'] == 'Query' and observation['observation']['outcome'] == {'VerifiedSuccess': {'checkpoint': checkpoint}}, 'Fresh reconcile did not verify the saved success')
    execution = observation['observation']['execution']
    require(execution['digest'] == execution['transaction']['digest'] == execution['effects']['transactionDigest'] == digest
            and base64.b64decode(execution['transaction']['bcs']['value'], validate=True).hex() == prepared['bcs_hex']
            and execution['effects']['status']['success'], 'Checkpoint bytes/digest/status mismatch')
    writes = [r['accumulatorWrite'] for r in execution['effects']['changedObjects']]
    require(len(writes) == 2 and {(w['address'], w['operation'], int(w['integerValue'])) for w in writes}
            == {(args.expected_sender, 'SPLIT', 10000), (registration['receiver'], 'MERGE', 10000)}
            and all(w['accumulatorType'] == '0x' + '0'*63 + '2::balance::Balance<' + ASSET + '>' for w in writes),
            'Exact 0.01 USDC typed debit/credit not confirmed')
    out = run(directory, 'accept-fault', ['accept', *merchant, '--out', str(receipt), '--fault-exit-after-credit-before-receipt'], expected=86)
    require(f'FAULT_EXIT_AFTER_CREDIT_BEFORE_RECEIPT digest={digest} invoice_hash={invoice_hash}' in out, 'Credit fault marker missing')
    mc = {'testnet_merchant_observations_v1': 1, 'testnet_credits_v1': 1}
    credited = state('after-credit-fault', wc, mc)
    require(not os.path.lexists(receipt) and not credited['wallet']['records']['testnet_confirmations_v1'], 'Receipt/association appeared before recovery')
    stored = one(credited['merchant'], 'testnet_credits_v1')
    # A strict byte golden for the existing native serde pretty export, not chain encoding.
    expected_receipt = json.dumps(stored, indent=2, ensure_ascii=False).encode()
    run(directory, 'accept-restart', ['accept', *merchant, '--out', str(receipt)])
    require(read(receipt) == expected_receipt, 'Regenerated receipt differs from committed receipt bytes')
    state('after-accept-restart', wc, mc)
    run(directory, 'confirm', ['confirm', *wallet, '--receipt', str(receipt)])
    wc.update(testnet_confirmations_v1=1, testnet_wallet_observations_v1=wc['testnet_wallet_observations_v1'] + 1)
    associated = state('after-confirm', wc, mc)
    require(one(associated['wallet'], 'testnet_confirmations_v1') == stored, 'Wallet association differs from committed credit')
    out = run(directory, 'accept-duplicate', ['accept', *merchant, '--out', str(replayed)])
    require('MERCHANT_EXISTING_IDENTICAL_CREDIT' in out and read(replayed) == read(receipt) == expected_receipt, 'Duplicate credit/receipt mismatch')
    out = run(directory, 'confirm-duplicate', ['confirm', *wallet, '--receipt', str(replayed)])
    require('WALLET_EXISTING_IDENTICAL_CHAIN_CONFIRMED_ASSOCIATION' in out, 'Duplicate association mismatch')
    final = state('after', wc, mc)
    return {'result': 'PASS_REAL_TESTNET_APPLICATION_BOUNDARY_RECOVERY', 'chain_submission': True,
            'digest': digest, 'invoice_hash': invoice_hash, 'checkpoint': checkpoint, 'amount_base_units': 10000,
            'faults': ['received_success_before_observation', 'committed_credit_before_receipt'],
            'new_dispatch_attempts': 1, 'new_merchant_credits': 1, 'new_wallet_associations': 1,
            'merchant_credits_before': before['merchant']['counts']['testnet_credits_v1'],
            'merchant_credits_after': final['merchant']['counts']['testnet_credits_v1'],
            'original_artifacts_unchanged': True, 'receipt_bytes_identical': True, 'transport': transport,
            'limits': 'Chosen fullnodes, not light-client/operator independence; no dropped RPC packet, RF, power-cut, custody or authenticated business ACK proof'}


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--trial-directory', type=Path, required=True)
    parser.add_argument('--merchant-ledger', type=Path, required=True)
    parser.add_argument('--preserve-credit-digest', required=True)
    parser.add_argument('--merchant-key', required=True)
    parser.add_argument('--expected-sender', required=True)
    parser.add_argument('--endpoint', required=True)
    parser.add_argument('--merchant-endpoint', required=True)
    parser.add_argument('--evidence-parent', type=Path, required=True)
    parser.add_argument('--pcsc-result', type=Path, help='Optional existing fresh-trial public transport result; never inferred')
    args = parser.parse_args()
    private(args.evidence_parent, True)
    directory = Path(tempfile.mkdtemp(prefix='testnet-recovery-', dir=args.evidence_parent))
    with termination.installed():
        result_created = False
        try:
            print(f'NOT_DONE / evidence retained: {directory}', flush=True)
            try:
                result = execute(args, directory)
            finally:
                with termination.defer():
                    ledger = directory / 'processes.jsonl'
                    identities = [json.loads(line) for line in ledger.read_text().splitlines()] if ledger.exists() else []
                    remaining = []
                    for row in identities:
                        p = Path(f'/proc/{row["pid"]}/stat')
                        if p.exists() and p.read_text().rsplit(')', 1)[1].split()[19] == row['start_ticks']:
                            remaining.append(row)
                    save(directory, 'cleanup.json', {'remaining_owned_processes': remaining})
                    require(not remaining, 'Owned runtime remains; no PASS')
                termination.check()
            result['owned_runtimes_remaining'] = 0
            with termination.defer():
                with (directory / 'result.json').open('x') as output:
                    result_created = True
                    json.dump(result, output, indent=2)
                    output.write('\n')
            termination.check()
            print(f'PASS / {directory}/result.json', flush=True)
        except BaseException as error:
            with termination.defer():
                if result_created:
                    (directory / 'result.json').unlink(missing_ok=True)
                signum = termination.pending
                save(directory, 'failure.json', {'result': 'STOPPED_NOT_DONE', 'error': str(error),
                     'signal': signum, 'exit_code': 128 + signum if signum is not None else 1,
                     'notice': 'Possible broadcast/effects remain unknown. Retain journals and reconcile the saved digest; never re-sign, create a new operation or automatically rerun fault/submit.'})
            if signum is not None and not isinstance(error, TrialStopped):
                raise TrialStopped(signum) from error
            raise


if __name__ == '__main__':
    try:
        main()
    except TrialStopped as error:
        print(f'STOPPED_NOT_DONE: {error}. Reconcile the saved digest; never re-sign or create a new operation.', file=sys.stderr)
        raise SystemExit(128 + error.signum)
    except (OSError, RuntimeError, ValueError, KeyError, sqlite3.Error, subprocess.SubprocessError) as error:
        raise SystemExit(f'STOPPED_NOT_DONE: {error}. Retain journals; no automatic retry.')
