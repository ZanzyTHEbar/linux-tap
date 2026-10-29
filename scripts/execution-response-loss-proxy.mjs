// FUTURE HUMAN-OPERATED TESTNET TRIAL CANDIDATE. Not a wallet, signer, or unattended submitter.
// Independent of the proven READ-ONLY query proxy: that component is not modified.
import http2 from 'node:http2';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { spawn, spawnSync } from 'node:child_process';
import { parseArgs } from 'node:util';
import { createInterface } from 'node:readline/promises';

const UPSTREAM = 'https://fullnode.testnet.sui.io:443';
const PROFILE = 'circle-sui-testnet/execution-response-loss-seal/v1';
const CHAIN = '69WiPg3DAQiwdxfncX6wYQ2siKwAe6L9BZthQea3JNMD';
const ASSET = '0xa1ec7fc00a6f40db9693ad1415d0c193ad3906494428cf252621037bd7117e29::usdc::USDC';
const EXEC = '/sui.rpc.v2.TransactionExecutionService/ExecuteTransaction';
const SIM = '/sui.rpc.v2.TransactionExecutionService/SimulateTransaction';
const SUB = '/sui.rpc.v2.SubscriptionService/SubscribeCheckpoints';
const READS = new Set([
  '/sui.rpc.v2.LedgerService/GetServiceInfo', '/sui.rpc.v2.LedgerService/GetEpoch',
  '/sui.rpc.v2.LedgerService/GetTransaction', '/sui.rpc.v2.StateService/GetCoinInfo',
  '/sui.rpc.v2.StateService/GetBalance', SUB,
]);
const HEADERS = ['content-type', 'te', 'grpc-encoding', 'grpc-accept-encoding', 'grpc-timeout'];
const RESPONSE = ['content-type', 'grpc-encoding', 'grpc-accept-encoding', 'grpc-status', 'grpc-message'];
const sha = data => createHash('sha256').update(data).digest('hex');
const require = (condition, message) => { if (!condition) throw new Error(message); };
const { values: args } = parseArgs({ options: {
  directory: { type: 'string' }, port: { type: 'string', default: '0' },
  seal: { type: 'string' }, 'arm-execution-response-loss': { type: 'boolean', default: false },
  'offline-guard-lab': { type: 'boolean', default: false },
  'preparation-inbox': { type: 'string' }, 'merchant-key': { type: 'string' },
  'expected-sender': { type: 'string' },
}, strict: true, allowPositionals: false });
process.umask(0o077);
const offline = args['offline-guard-lab'];
require(!offline || (args.seal && !args['arm-execution-response-loss']),
  'Offline guard lab requires a seal and forbids human arming; no upstream connection is allowed');
function privateDirectory(name) {
  require(name && path.isAbsolute(name) && fs.realpathSync(name) === name, 'Canonical absolute private directory required');
  const stat = fs.lstatSync(name);
  require(stat.isDirectory() && stat.uid === process.getuid() && (stat.mode & 0o777) === 0o700, 'Expected owned mode-0700 directory');
}
function privateLocator(name) {
  require(typeof name === 'string', 'Private file locator required');
  privateDirectory(path.dirname(name));
  const stat = fs.lstatSync(name);
  require(fs.realpathSync(name) === name && stat.isFile() && stat.nlink === 1
    && stat.uid === process.getuid() && (stat.mode & 0o777) === 0o600,
  'Expected owned single-link mode-0600 file');
  return stat;
}
function readPrivate(name) {
  require(privateLocator(name).size <= 65_536, 'Private record exceeds bound');
  return fs.readFileSync(name);
}
privateDirectory(args.directory);
const at = name => path.join(args.directory, name);
function durableNew(name, bytes) {
  const fd = fs.openSync(at(name), 'wx', 0o600);
  try { fs.writeFileSync(fd, bytes); fs.fsyncSync(fd); } finally { fs.closeSync(fd); }
  const parent = fs.openSync(args.directory, 'r');
  try { fs.fsyncSync(parent); } finally { fs.closeSync(parent); }
}
function bindOnce(name, bytes) {
  if (fs.existsSync(at(name))) require(readPrivate(at(name)).equals(bytes), 'Existing operation binding differs; never reset it');
  else durableNew(name, bytes);
}
// Standard unary gRPC framing only. Protobuf/BCS/crypto are native official-SDK responsibilities.
function frame(hex) {
  require(typeof hex === 'string' && /^[0-9a-f]+$/.test(hex) && hex.length % 2 === 0
    && hex.length <= 65_536, 'Invalid canonical protobuf hex');
  const payload = Buffer.from(hex, 'hex');
  const header = Buffer.alloc(5);
  header.writeUInt32BE(payload.length, 1);
  return Buffer.concat([header, payload]);
}
let port = Number(args.port);
require(Number.isInteger(port) && (port === 0 || (port >= 1024 && port <= 65535)), 'Invalid loopback port');
let previous;
if (fs.existsSync(at('endpoint.json'))) {
  previous = JSON.parse(readPrivate(at('endpoint.json')));
  require(previous.upstream === UPSTREAM && /^http:\/\/127\.0\.0\.1:[0-9]+$/.test(previous.endpoint), 'Invalid stored endpoint');
  const stored = Number(new URL(previous.endpoint).port);
  require(stored >= 1024 && stored <= 65535 && (port === 0 || port === stored), 'Restart must use the SAME pinned endpoint');
  port = stored;
}
let seal, executeFrame, simulationFrame, sealHash;
if (args.seal) {
  const bytes = readPrivate(args.seal);
  seal = JSON.parse(bytes);
  const keys = 'profile upstream endpoint genesis sender receiver merchant_pin invoice_hash digest artifact_tag asset amount_base_units execution_protobuf_hex simulation_protobuf_hex authority wallet_journal invoice_inbox registration'.split(' ');
  require(Object.keys(seal).length === keys.length && keys.every(k => Object.hasOwn(seal, k)), 'Unexpected seal fields');
  require(previous && seal.endpoint === previous.endpoint && seal.profile === PROFILE && seal.upstream === UPSTREAM
    && seal.genesis === CHAIN && seal.asset === ASSET && seal.amount_base_units === 10_000,
  'Seal must bind the existing endpoint and exact Circle testnet profile/amount');
  require(['sender', 'receiver'].every(k => /^0x[0-9a-f]{64}$/.test(seal[k]))
    && seal.sender !== seal.receiver
    && ['merchant_pin', 'invoice_hash', 'artifact_tag'].every(k => /^[0-9a-f]{64}$/.test(seal[k]))
    && /^[1-9A-HJ-NP-Za-km-z]{32,64}$/.test(seal.digest)
    && seal.authority === 'BINDING_ONLY_NOT_SUBMISSION_APPROVAL; native TESTNET SUBMIT and separate human proxy arming required',
  'Invalid native seal binding');
  for (const name of ['wallet_journal', 'invoice_inbox', 'registration']) privateLocator(seal[name]);
  executeFrame = frame(seal.execution_protobuf_hex);
  simulationFrame = frame(seal.simulation_protobuf_hex);
  sealHash = sha(bytes);
  bindOnce('bound-seal.json', bytes);
}
if (args['preparation-inbox']) {
  readPrivate(args['preparation-inbox']);
  require(/^[0-9a-f]{64}$/.test(args['merchant-key']) && /^0x[0-9a-f]{64}$/.test(args['expected-sender']),
    'Preparation needs explicit public pin and sender');
}
let armed = false;
if (args['arm-execution-response-loss']) {
  require(seal && !fs.existsSync(at('execute-forwarded.json')), 'Unconsumed native seal required; never rearm/replay');
  require(process.stdin.isTTY && process.stderr.isTTY, 'Arming requires a human terminal; no injected approval');
  console.error(`TESTNET ONLY: at most ONE sealed 0.01-USDC Execute may be forwarded.\nDigest: ${seal.digest}\nReceiver: ${seal.receiver}\nNative TESTNET SUBMIT consent remains required. No permission follows from a seal alone.`);
  const terminal = createInterface({ input: process.stdin, output: process.stderr });
  try { armed = (await terminal.question('Type ARM ONE TESTNET EXECUTION RESPONSE LOSS: ')).trim() === 'ARM ONE TESTNET EXECUTION RESPONSE LOSS'; }
  finally { terminal.close(); }
  require(armed, 'Arming declined');
}
const traceName = at('execution-proxy-events.jsonl');
if (fs.existsSync(traceName)) require(fs.lstatSync(traceName).size <= 524_288 && readPrivate(traceName).length <= 65_536,
  'Retain oversized trace; do not truncate/reset operation');
const traceFd = fs.openSync(traceName, 'a', 0o600);
let stopping = false, requests = 0, eventCount = 0;
function record(event) {
  if (stopping) return;
  require(++eventCount <= 512, 'Trace event budget exceeded');
  fs.writeSync(traceFd, JSON.stringify({ utc: new Date().toISOString(), ...event }) + '\n');
}
const sessions = new Set(), upstreams = new Set(), nativeChecks = new Set();
const server = http2.createServer({ maxHeaderListPairs: 32, settings: { maxConcurrentStreams: 8 } });
async function stop(code, reason) {
  if (stopping) return;
  stopping = true;
  try { fs.writeSync(traceFd, JSON.stringify({ event: 'STOPPED', reason, code, requests,
    possible_effect: !offline && fs.existsSync(at('execute-forwarded.json')) }) + '\n'); } catch {}
  server.close();
  for (const session of [...sessions, ...upstreams]) session.destroy();
  const waiting = [...nativeChecks].map(child => new Promise(resolve => child.once('close', resolve)));
  for (const child of nativeChecks) child.kill('SIGTERM');
  const escalation = setTimeout(() => { for (const child of nativeChecks) child.kill('SIGKILL'); }, 1000);
  const deadline = setTimeout(() => process.exit(1), 4000);
  await Promise.all(waiting);
  clearTimeout(escalation); clearTimeout(deadline);
  try {
    fs.writeSync(traceFd, JSON.stringify({ event: 'OWNED_CLEANUP_COMPLETE', native_predicates_remaining: nativeChecks.size,
      waited: true }) + '\n');
    fs.fsyncSync(traceFd); fs.closeSync(traceFd);
  } finally { process.exit(code); }
}
for (const signal of ['SIGTERM', 'SIGINT', 'SIGHUP']) process.on(signal, () => stop(0, signal));
server.on('error', () => stop(1, 'bind/transport failure; no alternate port'));
process.on('uncaughtException', () => stop(1, 'fatal gateway error; retain gate and reconcile'));
process.on('unhandledRejection', () => stop(1, 'fatal gateway error; retain gate and reconcile'));
setTimeout(() => stop(1, '15-minute operator-session deadline; retain state and reconcile'), 900_000).unref();
server.on('session', session => {
  sessions.add(session); session.setTimeout(90_000, () => session.destroy());
  session.on('error', () => {}); session.on('close', () => sessions.delete(session));
});
function reject(stream, id, reason) {
  record({ event: 'REJECTED_NO_FORWARD', id, reason });
  if (!stream.closed) { stream.respond({ ':status': 403 }); stream.end(); }
}
const nativeBinary = fileURLToPath(new URL('../target/debug/linux-tap', import.meta.url));
function nativePredicate(command) {
  return spawnSync(nativeBinary, ['testnet', ...command],
    { input: '', timeout: 5000, killSignal: 'SIGKILL', maxBuffer: 8192 });
}
function nativeDispatchPredicate(id) {
  return new Promise(resolve => {
    // Async so SDK ledger/checkpoint proof is never parked behind this approval check.
    const child = spawn(nativeBinary, ['testnet', 'check-execution-seal', '--seal', at('bound-seal.json')],
      { stdio: ['ignore', 'ignore', 'ignore'] });
    nativeChecks.add(child);
    let startTicks = null;
    try { startTicks = fs.readFileSync(`/proc/${child.pid}/stat`, 'utf8').split(') ').at(-1).split(' ')[19]; } catch {}
    record({ event: 'OWNED_NATIVE_CHECK_STARTED', id, pid: child.pid, start_ticks: startTicks });
    const timer = setTimeout(() => child.kill('SIGTERM'), 5000);
    const kill = setTimeout(() => child.kill('SIGKILL'), 6000);
    child.on('error', () => {});
    child.on('close', (status, signal) => {
      clearTimeout(timer); clearTimeout(kill); nativeChecks.delete(child);
      record({ event: 'NATIVE_FIRST_DISPATCH_PREDICATE', id, exit_code: status, signal, waited: true });
      resolve(status);
    });
  });
}
server.on('stream', (downstream, headers) => {
  downstream.on('error', () => {});
  const id = ++requests, method = headers[':path'];
  let resetQueued = false;
  downstream.once('close', () => {
    if (method === EXEC && !resetQueued) record({ event: 'FAULT_NOT_REACHED', id,
      reason: 'Execute stream closed without a proxy reset; never infer that a fault was applied' });
  });
  if (requests > 64 || headers[':method'] !== 'POST' || (!READS.has(method) && method !== EXEC && method !== SIM)
      || headers.authorization || headers.cookie || headers['proxy-authorization']
      || !String(headers['content-type']).startsWith('application/grpc')) {
    reject(downstream, id, 'method/credentials/budget'); return;
  }
  if (method === EXEC && ((!armed && !offline) || !seal || fs.existsSync(at('execute-forwarded.json')))) {
    reject(downstream, id, 'UNARMED_OR_CONSUMED: no execution'); return;
  }
  let size = 0; const chunks = [];
  downstream.setTimeout(90_000, () => downstream.close(http2.constants.NGHTTP2_CANCEL));
  downstream.on('data', chunk => {
    size += chunk.length;
    if (size > 32_773) downstream.close(http2.constants.NGHTTP2_CANCEL);
    else chunks.push(chunk);
  });
  downstream.on('end', async () => {
    if (stopping || downstream.closed || size > 32_773) return;
    const body = Buffer.concat(chunks);
    const unary = body.length >= 5 && body[0] === 0 && body.readUInt32BE(1) === body.length - 5
      && (!headers['grpc-encoding'] || headers['grpc-encoding'] === 'identity');
    if (!unary) { reject(downstream, id, 'compressed/multiple/malformed frame'); return; }
    if (method === EXEC && !body.equals(executeFrame)) { reject(downstream, id, 'exact Execute seal mismatch'); return; }
    if (method === SIM) {
      if (seal) {
        if (!body.equals(simulationFrame)) { reject(downstream, id, 'exact checked simulation mismatch'); return; }
      } else {
        if (!args['preparation-inbox']) { reject(downstream, id, 'native preparation context required'); return; }
        const check = nativePredicate(['check-lab-simulation', '--invoice-inbox', args['preparation-inbox'],
          '--merchant-key', args['merchant-key'], '--expected-sender', args['expected-sender'],
          '--request-hex', body.subarray(5).toString('hex')]);
        record({ event: 'NATIVE_SIMULATION_PREDICATE', id, exit_code: check.status });
        if (check.status !== 0) { reject(downstream, id, 'native checked-simulation predicate failed'); return; }
      }
    }
    const requestHash = sha(body);
    if (method === EXEC) {
      // Seal alone is insufficient: the real native first-dispatch barrier must already exist.
      if (sha(readPrivate(at('bound-seal.json'))) !== sealHash) { reject(downstream, id, 'bound seal changed'); return; }
      const approval = await nativeDispatchPredicate(id);
      if (stopping || downstream.closed) return;
      if (approval !== 0 || sha(readPrivate(at('bound-seal.json'))) !== sealHash) {
        reject(downstream, id, 'native approval/intent/attempt/seal predicate failed'); return;
      }
      try {
        // Consumed BEFORE any upstream connection/stream. Failures never erase or reset this gate.
        durableNew('execute-forwarded.json', JSON.stringify({ profile: PROFILE, digest: seal.digest,
          invoice_hash: seal.invoice_hash, seal_sha256: sealHash, request_sha256: requestHash,
          state: offline ? 'OFFLINE_CONSUMED_NO_UPSTREAM' : 'CONSUMED_BEFORE_NETWORK_POSSIBLE_EFFECT',
          utc: new Date().toISOString() }) + '\n');
      } catch { reject(downstream, id, 'consumed/unwritable durable gate'); return; }
      armed = false;
    }
    // This branch is unconditional for EVERY method in offline mode, before http2.connect.
    // It exercises the real framing/seal/durable gate but never fabricates chain/RPC success.
    if (offline) {
      reject(downstream, id, method === EXEC ? 'OFFLINE_EXACT_SEAL_CONSUMED_NO_FORWARD' : 'OFFLINE_NO_UPSTREAM');
      return;
    }
    record({ event: 'UPSTREAM_REQUEST_STARTED', id, method, request_sha256: requestHash, request_bytes: body.length });
    const upstream = http2.connect(UPSTREAM, { rejectUnauthorized: true }); upstreams.add(upstream);
    let complete = false;
    const timer = setTimeout(() => fail('upstream whole-operation deadline'), method === SUB ? 75_000 : 20_000);
    function fail(reason) {
      if (complete || stopping) return;
      complete = true; clearTimeout(timer);
      record({ event: method === EXEC ? 'FAULT_NOT_REACHED_POSSIBLE_EFFECT' : 'UPSTREAM_INCOMPLETE', id, method, reason });
      upstream.destroy(); if (!downstream.closed) downstream.close(http2.constants.NGHTTP2_CANCEL);
    }
    upstream.on('error', () => fail('TLS/HTTP2 failure'));
    upstream.on('close', () => upstreams.delete(upstream));
    const outgoing = { ':method': 'POST', ':path': method };
    for (const name of HEADERS) if (headers[name] !== undefined) outgoing[name] = headers[name];
    const request = upstream.request(outgoing);
    let responseHeaders, trailers = {}, responseBytes = 0;
    const responseChunks = [], responseHash = createHash('sha256');
    function replyHeaders() {
      const output = { ':status': responseHeaders[':status'] };
      for (const name of RESPONSE) if (responseHeaders[name] !== undefined) output[name] = responseHeaders[name];
      downstream.respond(output, { waitForTrailers: true });
      downstream.on('wantTrailers', () => {
        if (downstream.destroyed) return;
        const output = {};
        for (const name of ['grpc-status', 'grpc-message']) if (trailers[name] !== undefined) output[name] = trailers[name];
        downstream.sendTrailers(output);
      });
    }
    request.on('error', () => fail('upstream stream error'));
    request.on('response', value => {
      responseHeaders = value;
      if (method !== EXEC && !downstream.closed) {
        replyHeaders(); request.pipe(downstream); // Reads/stream pass through with backpressure; no proof blackout or delay.
      }
    });
    request.on('trailers', value => { trailers = value; });
    request.on('data', chunk => {
      responseBytes += chunk.length; responseHash.update(chunk);
      if (responseBytes > (method === SUB ? 8_388_608 : 1_048_576)) fail('response byte bound');
      else if (method === EXEC) responseChunks.push(chunk);
    });
    downstream.on('aborted', () => {
      if (method === SUB && !complete && !stopping) {
        complete = true; clearTimeout(timer); upstream.destroy();
        record({ event: 'NATIVE_SUBSCRIPTION_CANCELLED', id, method });
      } else fail('client cancelled before completion');
    });
    request.on('end', () => {
      if (complete || stopping) return;
      complete = true; clearTimeout(timer); upstream.close();
      const status = String(trailers['grpc-status'] ?? responseHeaders?.['grpc-status']);
      record({ event: 'UPSTREAM_RESPONSE_COMPLETE', id, method, http_status: responseHeaders?.[':status'],
        grpc_status: status, bytes: responseBytes, sha256: responseHash.digest('hex') });
      if (method !== EXEC) return;
      const response = Buffer.concat(responseChunks);
      if (method === EXEC) {
        durableNew(`execute-response-${id}.body`, response);
        durableNew(`execute-response-${id}.json`, JSON.stringify({ headers: responseHeaders, trailers }) + '\n');
        if (downstream.closed || responseHeaders?.[':status'] !== 200 || status !== '0' || responseBytes < 5) {
          record({ event: 'FAULT_NOT_REACHED_POSSIBLE_EFFECT', id, reason: 'client cancelled or upstream not complete gRPC success' });
          if (!downstream.closed) { replyHeaders(); downstream.end(response); }
          return;
        }
        downstream.once('close', () => record({ event: 'EXECUTE_RESET_SENT_NEEDS_NATIVE_TRACE', id,
          rst_code: downstream.rstCode, downstream_success_headers_sent: false, downstream_response_bytes_sent: 0,
          acceptance: 'NOT_PASS_WITHOUT_NATIVE_TRANSPORT_ERROR_AND_MATCHING_CHECKPOINT_PROOF' }));
        resetQueued = true;
        downstream.close(http2.constants.NGHTTP2_CANCEL);
      }
    });
    request.end(body);
  });
});
server.listen(port, '127.0.0.1', () => {
  const endpoint = `http://127.0.0.1:${server.address().port}`;
  bindOnce('endpoint.json', Buffer.from(JSON.stringify({ endpoint, upstream: UPSTREAM }) + '\n'));
  record({ event: 'READY', endpoint, mode: offline ? 'OFFLINE_GUARDS_NO_UPSTREAM' : armed ? 'HUMAN_ARMED_ONE_SEALED_EXECUTE' : 'UNARMED_NO_EXECUTE',
    seal_sha256: sealHash ?? null, request_sha256: executeFrame ? sha(executeFrame) : null });
  console.log(JSON.stringify({ endpoint, armed, notice: 'CANDIDATE; never delete/reset bindings or gate; not a paid-status authority' }));
});
