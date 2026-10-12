import { test } from 'node:test';
import assert from 'node:assert/strict';
import { releaseRequest } from './release-request.mjs';
const revision = 'a'.repeat(40), manifest = '[package]\nversion = "0.1.3"\n';
test('only an authorized exact source/version request can promote a release', () => {
  assert.deepEqual(releaseRequest(`/release v0.1.3 ${revision}`, 'coela-oss', revision, manifest), { tag: 'v0.1.3', revision });
  for (const [request, actor, source] of [
    [`/release v0.1.3 ${revision}`, 'external-contributor', revision],
    [`/release v0.1.3 ${revision}`, 'coela-oss', 'b'.repeat(40)],
    [`/release v0.1.3 ${revision}\n`, 'coela-oss', revision],
    [`/release v0.1.2 ${revision}`, 'coela-oss', revision],
    [`/release v0.1.3 ${revision}\nmalicious`, 'coela-oss', revision],
    [`/release v0.1.3 $()`, 'coela-oss', revision],
  ]) assert.throws(() => releaseRequest(request, actor, source, manifest));
});
