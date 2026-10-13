import { test } from 'node:test';
import assert from 'node:assert/strict';
import { releaseRequest } from './release-request.mjs';
const revision = 'a'.repeat(40), manifest = '[package]\nversion = "0.1.4"\n';
test('only an authorized exact source/version request can promote a release', () => {
  assert.deepEqual(releaseRequest(`/release security-inspector-v0.1.4 ${revision}`, 'coela-oss', revision, manifest), { tag: 'security-inspector-v0.1.4', revision });
  for (const [request, actor, source] of [
    [`/release v0.1.4 ${revision}`, 'coela-oss', revision],
    [`/release security-inspector-v0.1.4 ${revision}`, 'external-contributor', revision],
    [`/release security-inspector-v0.1.4 ${revision}`, 'coela-oss', 'b'.repeat(40)],
    [`/release security-inspector-v0.1.4 ${revision}\n`, 'coela-oss', revision],
    [`/release security-inspector-v0.1.2 ${revision}`, 'coela-oss', revision],
    [`/release security-inspector-v0.1.4 ${revision}\nmalicious`, 'coela-oss', revision],
    [`/release security-inspector-v0.1.4 $()`, 'coela-oss', revision],
  ]) assert.throws(() => releaseRequest(request, actor, source, manifest));
});
