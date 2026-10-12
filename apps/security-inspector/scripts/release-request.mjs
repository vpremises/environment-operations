import { readFileSync, appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export function releaseRequest(request, actor, revision, manifest) {
  if (actor !== 'coela-oss') throw new Error('Unauthorized release requester');
  const match = /^\/release (v[0-9]+\.[0-9]+\.[0-9]+) ([a-f0-9]{40})$/.exec(request);
  const version = /^version = "([0-9]+\.[0-9]+\.[0-9]+)"$/m.exec(manifest)?.[1];
  if (!match || match[0] !== request || !version || match[1] !== `v${version}` || match[2] !== revision) {
    throw new Error('Release must select the declared version and exact current source');
  }
  return { tag: match[1], revision: match[2] };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.env.GITHUB_REF !== 'refs/heads/main') throw new Error('Main source required');
  const request = releaseRequest(process.env.RELEASE_REQUEST, process.env.GITHUB_ACTOR,
    process.env.GITHUB_SHA, readFileSync('Cargo.toml', 'utf8'));
  appendFileSync(process.env.GITHUB_OUTPUT, `tag=${request.tag}\nrevision=${request.revision}\n`);
}
