#!/usr/bin/env python3
"""Opt-in read-only RPC loss lab, using reconstructed public state, never a live wallet.

No preparation, signing, submission, merchant acceptance or real journal access.
The second of three identical native transaction queries loses its real upstream
response at a local HTTP/2 proxy, before tonic receives response headers or data.
"""
import argparse
from contextlib import closing
import copy
import importlib.util
import json
import os
from pathlib import Path
import re
import select
import sqlite3
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
import demo

spec = importlib.util.spec_from_file_location('recovery', Path(__file__).with_name('testnet-recovery.py'))
recovery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recovery)
require, save, sha = recovery.require, recovery.save, recovery.sha
UPSTREAM = 'https://fullnode.testnet.sui.io:443'
DIGEST = '52wFhV654LoLhXycxB5nxn3B2ddM4siC7Mm72opYt9sW'
INVOICE = '46bb3906abb57c66c4a5cf6b748442475dfb366a2b159712f274c13479be4829'
CHECKPOINT = 400484678
TABLES = ('testnet_candidates_v1', 'testnet_reservations_v1', 'testnet_artifacts_v1',
          'testnet_dispatch_v1', 'testnet_attempts_v1', 'testnet_wallet_observations_v1')
# Same columns/constraints as Store. Native Store::open creates remaining empty tables.
SCHEMA = '''
PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
CREATE TABLE testnet_candidates_v1 (invoice_hash TEXT PRIMARY KEY, issuance TEXT NOT NULL UNIQUE, value TEXT NOT NULL CHECK(length(value)<=65536));
CREATE TABLE testnet_reservations_v1 (invoice_hash TEXT PRIMARY KEY REFERENCES testnet_candidates_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=65536));
CREATE TABLE testnet_artifacts_v1 (invoice_hash TEXT PRIMARY KEY REFERENCES testnet_reservations_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=65536));
CREATE TABLE testnet_dispatch_v1 (invoice_hash TEXT PRIMARY KEY REFERENCES testnet_candidates_v1(invoice_hash), digest TEXT NOT NULL UNIQUE, value TEXT NOT NULL CHECK(length(value)<=1048576));
CREATE TABLE testnet_attempts_v1 (id INTEGER PRIMARY KEY, invoice_hash TEXT NOT NULL REFERENCES testnet_dispatch_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=1048576));
CREATE TABLE testnet_wallet_observations_v1 (id INTEGER PRIMARY KEY, invoice_hash TEXT NOT NULL REFERENCES testnet_candidates_v1(invoice_hash), value TEXT NOT NULL CHECK(length(value)<=1048576));
PRAGMA foreign_keys=ON;
'''


def captured_rows(source):
    # Only the already extracted public verifier snapshot; never discover/open live state.
    require(source.name == 'before-fresh-query.json'
            and source.parent.parent == Path('/tmp/opencode')
            and source.parent.name.startswith('linux-tap-recovery-verification-'),
            'Use the retained public verifier snapshot under /tmp/opencode')
    raw = recovery.read(source)
    data = json.loads(raw)['wallet']['records']
    require(all(len(data[table]) == 1 for table in TABLES[:-1]), 'Expected one original row per issuance/dispatch table')
    rows = {table: copy.deepcopy(data[table][:1]) for table in TABLES}
    require(all(len(value) == 1 for value in rows.values()), 'Missing original public records')
    candidate = json.loads(rows[TABLES[0]][0]['value'])
    artifact = json.loads(rows['testnet_artifacts_v1'][0]['value'])
    require(candidate['digest'] == artifact['digest'] == DIGEST
            and candidate['core_hash'] == INVOICE and candidate['endpoint'] == UPSTREAM
            and candidate['bcs_hex'] == artifact['bcs_hex'], 'Wrong retained payment')
    observation = json.loads(rows['testnet_wallet_observations_v1'][0]['value'])
    require(observation['source'] == 'Query' and observation['observation']['outcome'] == 'NotFound',
            'Reconstruction needs the pre-success Query/NotFound row')
    require(rows['testnet_attempts_v1'][0]['id'] == 1, 'Expected original single attempt')
    return rows, candidate, sha(raw)


def reconstruct(database, original, endpoint):
    rows = copy.deepcopy(original)
    for table in ('testnet_candidates_v1', 'testnet_dispatch_v1', 'testnet_attempts_v1'):
        value = json.loads(rows[table][0]['value'])
        require(value['endpoint'] == UPSTREAM, 'Unexpected source endpoint')
        value['endpoint'] = endpoint
        rows[table][0]['value'] = json.dumps(value, separators=(',', ':'))
    record = json.loads(rows['testnet_wallet_observations_v1'][0]['value'])
    record['observation']['endpoint'] = endpoint
    rows['testnet_wallet_observations_v1'][0]['value'] = json.dumps(record, separators=(',', ':'))
    with database.open('xb'):
        pass
    with closing(sqlite3.connect(database)) as db:
        db.executescript(SCHEMA)
        for table in TABLES:
            for row in rows[table]:
                columns = tuple(row)
                require(set(columns) <= {'invoice_hash', 'issuance', 'value', 'digest', 'id'}, 'Unexpected row columns')
                db.execute(f'INSERT INTO {table} ({",".join(columns)}) VALUES ({",".join("?" for _ in columns)})',
                           tuple(row.values()))
        db.commit()
    return rows


def facts(database):
    with closing(sqlite3.connect(database.as_uri() + '?mode=ro', uri=True)) as db:
        db.row_factory = sqlite3.Row
        names = {row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        # Missing initial tables are empty; native schema initialization adds them, not rows.
        return {table: [dict(row) for row in db.execute(f'SELECT * FROM {table} ORDER BY rowid')]
                if table in names else [] for table in recovery.TABLES}


def preserve(before, after, observations):
    for table in before:
        if table == 'testnet_wallet_observations_v1':
            require(after[table][:len(before[table])] == before[table]
                    and len(after[table]) == len(before[table]) + observations, 'Wrong observation delta')
        else:
            require(after[table] == before[table], f'Changed non-observation rows: {table}')


def last_observation(snapshot):
    record = json.loads(snapshot['testnet_wallet_observations_v1'][-1]['value'])
    require(record['source'] == 'Query', 'Not a keyless query observation')
    return record['observation']


def execute(source, directory):
    original, candidate, source_hash = captured_rows(source)
    proxy = None
    with (directory / 'proxy.stderr').open('xb') as error:
        try:
            with recovery.termination.defer():
                proxy = demo.launch(directory, 'readonly-http2-proxy',
                                    ['node', str(Path(__file__).with_name('query-response-loss-proxy.mjs')),
                                     '--directory', str(directory)], stdin=subprocess.DEVNULL,
                                    stdout=subprocess.PIPE, stderr=error)
            recovery.termination.check()
            require(select.select([proxy.stdout], [], [], 5)[0], 'Proxy did not become ready')
            ready = proxy.stdout.readline(1024)
            require(ready.endswith(b'\n'), 'Incomplete readiness record')
            (directory / 'proxy.stdout').write_bytes(ready)
            endpoint = json.loads(ready)['endpoint']
            require(re.fullmatch(r'http://127\.0\.0\.1:[1-9][0-9]{0,4}', endpoint), 'Not explicit loopback')
            control, pending = directory / 'control.sqlite', directory / 'pending.sqlite'
            reconstruct(control, original, endpoint)
            transformed = reconstruct(pending, original, endpoint)
            inbox = directory / 'inbox.json'
            inbox.write_bytes(bytes.fromhex(candidate['envelope_hex']))
            save(directory, 'provenance.json', {
                'profile': 'RECONSTRUCTED_LAB_STATE_NOT_LIVE_CUSTODY_OR_NEW_APPROVAL',
                'source_snapshot_sha256': source_hash, 'digest': DIGEST,
                'copied_rows': {table: len(rows) for table, rows in original.items()},
                'changes': ['wallet candidate/intent/attempt endpoint -> private loopback',
                            'original Query/NotFound endpoint -> private loopback',
                            'later historical success/association rows excluded'],
                'signed_artifact_value_unchanged': transformed['testnet_artifacts_v1'] == original['testnet_artifacts_v1'],
                'merchant_registration_and_endpoint_unchanged': True,
                'approval_authority': 'Reconstructed historical metadata only, not a new human approval',
                'upstream': UPSTREAM, 'endpoint': endpoint})
            intent = json.loads(original['testnet_dispatch_v1'][0]['value'])
            common = ['reconcile', '--invoice-inbox', str(inbox), '--merchant-key', candidate['merchant_pin'],
                      '--expected-sender', intent['identity']['sender'], '--endpoint', endpoint,
                      '--merchant-endpoint', UPSTREAM]
            before = facts(pending)
            save(directory, 'before.json', before)
            recovery.run(directory, 'control-query', [*common, '--journal', str(control)])
            control_result = facts(control)
            require(last_observation(control_result)['outcome'] == {'VerifiedSuccess': {'checkpoint': CHECKPOINT}},
                    'Control did not verify original checkpoint')
            save(directory, 'control.json', control_result)
            recovery.run(directory, 'lost-query-response', [*common, '--journal', str(pending)], expected=1)
            lost = facts(pending)
            preserve(before, lost, 1)
            observation = last_observation(lost)
            require(observation['outcome'] == 'Unknown' and observation['execution'] is None
                    and 'lookup failed' in observation['error'], 'Native client did not persist RPC Unknown')
            save(directory, 'after-loss.json', lost)
            recovery.run(directory, 'restart-query', [*common, '--journal', str(pending)])
            recovered = facts(pending)
            preserve(before, recovered, 2)
            require(last_observation(recovered)['outcome'] == {'VerifiedSuccess': {'checkpoint': CHECKPOINT}},
                    'Fresh process did not recover the exact saved checkpoint')
            require(last_observation(recovered)['execution'] == last_observation(control_result)['execution'],
                    'Control and recovery executions differ')
            require(len(recovered['testnet_artifacts_v1']) == len(recovered['testnet_dispatch_v1'])
                    == len(recovered['testnet_attempts_v1']) == 1
                    and not recovered['testnet_credits_v1'] and not recovered['testnet_confirmations_v1'],
                    'Unexpected lab artifact/attempt/credit/association count')
            save(directory, 'after-recovery.json', recovered)
        finally:
            with recovery.termination.defer():
                if proxy is not None:
                    if proxy.poll() is None:
                        proxy.terminate()
                    try:
                        proxy.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        proxy.kill()
                        proxy.wait(timeout=5)
                    proxy.stdout.close()
                    save(directory, 'proxy-exit.json', {'exit_code': proxy.returncode, 'waited': True})
            recovery.termination.check()
    require(proxy.returncode == 0, 'Proxy failed or exceeded deadline; do not retry automatically')
    events = [json.loads(line) for line in (directory / 'proxy-events.jsonl').read_text().splitlines()]
    requests = [event for event in events if event['event'] == 'UPSTREAM_READ_COMPLETE']
    faults = [event for event in events if event['event'] == 'fault_after_upstream_read_before_downstream_delivery']
    require(len(requests) == 6 and len(faults) == 1 and faults[0]['downstream_reset_code'] == 8
            and faults[0]['downstream_response_bytes_sent'] == 0, 'Not the intended real HTTP2 response-loss fault')
    require(not any(e['event'] in ('UPSTREAM_INCOMPLETE', 'CLIENT_CLOSED_EARLY') for e in events),
            'Unexpected transport failure, not a qualified run')
    identities = [json.loads(line) for line in (directory / 'processes.jsonl').read_text().splitlines()]
    for identity in identities:
        stat = Path(f"/proc/{identity['pid']}/stat")
        require(not stat.exists() or stat.read_text().rsplit(')', 1)[1].split()[19] != identity['start_ticks'],
                'An owned process identity remains')
    save(directory, 'result.json', {'result': 'PASS_READ_QUERY_RESPONSE_LOSS_ONLY', 'digest': DIGEST,
         'checkpoint': CHECKPOINT, 'reconstructed_lab_state': True, 'native_stage_exits': [0, 1, 0],
         'observations_before_loss_recovery': [1, 2, 3], 'artifacts': 1, 'intents': 1, 'attempts': 1,
         'credits': 0, 'associations': 0, 'upstream_read_requests': 6, 'upstream_mutating_requests': 0,
         'fault': faults[0], 'owned_runtimes_remaining': 0,
         'new_signature_or_submission': False, 'execution_response_loss_tested': False,
         'host_crash_or_power_cut_tested': False})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-read-only-network-lab', action='store_true', required=True)
    parser.add_argument('--public-snapshot', type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    # Validate the only source locator before starting any process or network activity.
    captured_rows(args.public_snapshot)
    directory = Path(tempfile.mkdtemp(prefix='linux-tap-query-loss-', dir='/tmp/opencode'))
    print(f'RECONSTRUCTED READ-ONLY LAB: {directory}', flush=True)
    with recovery.termination.installed():
        execute(args.public_snapshot, directory)
    print('PASS_READ_QUERY_RESPONSE_LOSS_ONLY; no execution response-loss or power-cut claim', flush=True)


if __name__ == '__main__':
    main()
