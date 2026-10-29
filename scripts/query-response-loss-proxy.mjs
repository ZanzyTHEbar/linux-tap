// Opt-in lab transport, not a wallet/chain engine. Never forwards execution or simulation.
import http2 from 'node:http2';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';

const UPSTREAM = 'https://fullnode.testnet.sui.io:443';
// Exact paths from pinned sui-rpc generated LedgerServiceClient, not inferred names.
const INFO = '/sui.rpc.v2.LedgerService/GetServiceInfo';
const TX = '/sui.rpc.v2.LedgerService/GetTransaction';
const methods = new Map([[INFO, 0], [TX, 0]]);
const MAX_BODY = 1_048_576;
const REQUEST_LIMIT = 65_536;
const FORWARD = ['content-type', 'te', 'grpc-encoding', 'grpc-accept-encoding', 'grpc-timeout'];
const RESPONSE = ['content-type', 'grpc-encoding', 'grpc-accept-encoding', 'grpc-status', 'grpc-message'];
const sha = data => createHash('sha256').update(data).digest('hex');
if (process.argv.length !== 4 || process.argv[2] !== '--directory') {
  throw new Error('Use --directory NEW_OWNED_PRIVATE_LAB_DIRECTORY');
}
const directory = process.argv[3];
const info = fs.lstatSync(directory);
if (!directory.startsWith('/tmp/opencode/') || fs.realpathSync(directory) !== directory
    || !info.isDirectory() || info.uid !== process.getuid() || (info.mode & 0o777) !== 0o700) {
  throw new Error('Lab requires canonical owned mode-0700 directory under /tmp/opencode');
}
process.umask(0o077);
const ledger = fs.openSync(path.join(directory, 'proxy-events.jsonl'), 'wx', 0o600);
let events = 0;
function record(value) {
  if (++events > 100) throw new Error('Event budget exceeded');
  fs.writeSync(ledger, JSON.stringify({ utc: new Date().toISOString(), ...value }) + '\n');
}
function save(name, bytes) {
  fs.writeFileSync(path.join(directory, name), bytes, { flag: 'wx', mode: 0o600 });
}
const downstreamSessions = new Set();
const upstreamSessions = new Set();
let stopping = false;
let requests = 0;
let transactionRequestHash;
const server = http2.createServer({ maxHeaderListPairs: 32, settings: { maxConcurrentStreams: 1 } });
function stop(code, reason) {
  if (stopping) return;
  stopping = true;
  record({ event: 'STOPPED', reason, code, upstream_methods: Object.fromEntries(methods) });
  server.close();
  for (const session of [...upstreamSessions, ...downstreamSessions]) session.destroy();
  fs.closeSync(ledger);
  process.exitCode = code;
  // Only this process's transport handles; no external PID signaling or service cleanup.
  setTimeout(() => process.exit(code), 50);
}
process.on('SIGTERM', () => stop(0, 'owned lab cleanup'));
process.on('SIGINT', () => stop(1, 'interrupted'));
process.on('SIGHUP', () => stop(1, 'interrupted'));
server.on('error', () => stop(1, 'server transport error'));
const lifetime = setTimeout(() => stop(1, '90-second total lab deadline'), 90_000);
lifetime.unref();
server.on('session', session => {
  downstreamSessions.add(session);
  session.setTimeout(12_000, () => session.destroy());
  session.on('error', () => {});
  session.on('close', () => downstreamSessions.delete(session));
});
server.on('stream', (downstream, headers) => {
  downstream.on('error', () => {});
  const method = headers[':path'];
  const id = ++requests;
  // Reject BEFORE creating an upstream connection, not merely before writing its body.
  if (stopping || id > 8 || headers[':method'] !== 'POST' || !methods.has(method)
      || headers.authorization || headers.cookie || headers['proxy-authorization']
      || !String(headers['content-type']).startsWith('application/grpc')) {
    record({ event: 'REJECTED_NO_FORWARD', id, method, upstream_created: false });
    downstream.respond({ ':status': 403 });
    downstream.end();
    return;
  }
  const chunks = [];
  let requestBytes = 0;
  downstream.setTimeout(10_000, () => downstream.close(http2.constants.NGHTTP2_CANCEL));
  downstream.on('data', chunk => {
    requestBytes += chunk.length;
    if (requestBytes > REQUEST_LIMIT) downstream.close(http2.constants.NGHTTP2_CANCEL);
    else chunks.push(chunk);
  });
  downstream.on('end', () => {
    if (downstream.closed || requestBytes > REQUEST_LIMIT || stopping) return;
    const body = Buffer.concat(chunks);
    const requestHash = sha(body);
    const index = methods.get(method) + 1;
    if (index > 3 || (method === TX && transactionRequestHash && requestHash !== transactionRequestHash)) {
      record({ event: 'REJECTED_NO_FORWARD', id, method, reason: 'budget or changed transaction query', upstream_created: false });
      downstream.close(http2.constants.NGHTTP2_CANCEL);
      return;
    }
    if (method === TX) transactionRequestHash ??= requestHash;
    methods.set(method, index);
    const upstream = http2.connect(UPSTREAM, { rejectUnauthorized: true });
    upstreamSessions.add(upstream);
    let complete = false;
    const timer = setTimeout(() => fail('8-second whole upstream request deadline'), 8_000);
    function fail(reason) {
      if (complete || stopping) return;
      complete = true;
      clearTimeout(timer);
      record({ event: 'UPSTREAM_INCOMPLETE', id, method, reason });
      upstream.destroy();
      if (!downstream.closed) downstream.close(http2.constants.NGHTTP2_CANCEL);
    }
    upstream.on('error', () => fail('TLS/HTTP2 error'));
    upstream.on('close', () => upstreamSessions.delete(upstream));
    const requestHeaders = { ':method': 'POST', ':path': method };
    for (const name of FORWARD) if (headers[name] !== undefined) requestHeaders[name] = headers[name];
    const request = upstream.request(requestHeaders);
    const responseChunks = [];
    let responseBytes = 0;
    let responseHeaders;
    let trailers = {};
    request.on('error', () => fail('upstream stream error'));
    request.on('response', value => { responseHeaders = value; });
    request.on('trailers', value => { trailers = value; });
    request.on('data', chunk => {
      responseBytes += chunk.length;
      if (responseBytes > MAX_BODY) fail('response exceeds bound');
      else responseChunks.push(chunk);
    });
    request.on('end', () => {
      if (complete || stopping) return;
      const grpcStatus = String(trailers['grpc-status'] ?? responseHeaders?.['grpc-status']);
      if (responseHeaders?.[':status'] !== 200 || grpcStatus !== '0' || responseBytes < 5) {
        fail('upstream did not return a complete successful gRPC response');
        return;
      }
      complete = true;
      clearTimeout(timer);
      const responseBody = Buffer.concat(responseChunks);
      save(`upstream-${id}.body`, responseBody);
      save(`upstream-${id}.json`, JSON.stringify({ method, headers: responseHeaders, trailers }, null, 2) + '\n');
      const facts = { id, method, index, request_sha256: requestHash, request_bytes: body.length,
        upstream_http_status: 200, upstream_grpc_status: grpcStatus,
        upstream_response_sha256: sha(responseBody), upstream_response_bytes: responseBytes };
      record({ event: 'UPSTREAM_READ_COMPLETE', ...facts });
      upstream.close();
      if (downstream.closed) {
        record({ event: 'CLIENT_CLOSED_EARLY', id });
        return;
      }
      if (method === TX && index === 2) {
        // No response headers or body have been delivered. The native SDK sees a real RST_STREAM.
        downstream.once('close', () => {
          if (!stopping) record({ event: 'fault_after_upstream_read_before_downstream_delivery',
            ...facts, downstream_response_bytes_sent: 0, downstream_reset_code: downstream.rstCode });
        });
        downstream.close(http2.constants.NGHTTP2_CANCEL);
      } else {
        const outputHeaders = { ':status': 200 };
        for (const name of RESPONSE) if (responseHeaders[name] !== undefined) outputHeaders[name] = responseHeaders[name];
        downstream.respond(outputHeaders, { waitForTrailers: true });
        downstream.on('wantTrailers', () => {
          const outputTrailers = {};
          for (const name of ['grpc-status', 'grpc-message']) if (trailers[name] !== undefined) outputTrailers[name] = trailers[name];
          downstream.sendTrailers(outputTrailers);
        });
        downstream.end(responseBody);
        record({ event: 'FORWARDED_UNMODIFIED_BODY', ...facts });
      }
    });
    request.end(body);
    downstream.on('aborted', () => fail('client aborted'));
  });
});
server.listen(0, '127.0.0.1', () => {
  const endpoint = `http://127.0.0.1:${server.address().port}`;
  record({ event: 'READY', endpoint, upstream: UPSTREAM, allowed_methods: [...methods.keys()],
    fault: 'second GetTransaction only; no downstream success headers/body before reset' });
  console.log(JSON.stringify({ endpoint }));
});
