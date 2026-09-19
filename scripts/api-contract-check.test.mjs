import { test } from 'node:test';
import assert from 'node:assert/strict';
import { checkContracts, routerOperations } from './api-contract-check.mjs';

const source = `Router::new()
  .route("/v1/skills/:id", get(detail).post(submit).layer(DefaultBodyLimit::max(123)))
  .route(
    "/v1/admin/skills/:id",
    axum::routing::delete(remove),
  ).with_state(state)`;
const square = `paths:
  /v1/skills/{skill_id}:
    get:
      security: []
    post:
      summary: Submit
`;
const admin = `paths:
  /v1/admin/skills/{id}:
    delete:
      summary: Remove
`;

test('matches multiline routes, chained methods, layers and differently named parameters', () => {
  assert.equal(checkContracts(source, [square, admin]), 3);
});
test('rejects an undocumented route and method on an existing route', () => {
  assert.throws(() => checkContracts(source + '.route("/v1/new", get(new_handler))', [square, admin]), /Missing OpenAPI: GET \/v1\/new/);
  assert.throws(() => checkContracts(source.replace('.post(submit)', '.post(submit).put(update)'), [square, admin]), /Missing OpenAPI: PUT/);
});
test('rejects obsolete documented operations and wrong HTTP methods', () => {
  assert.throws(() => checkContracts(source, [square, admin.replace('delete:', 'put:')]), error =>
    /Missing OpenAPI: DELETE/.test(error.message) && /Missing backend: PUT/.test(error.message));
});
test('ignores commented routes but rejects unsupported or missing registrations', () => {
  assert.equal(checkContracts('// .route("/ignored", get(fake))\n' + source, [square, admin]), 3);
  for (const input of ['Router::new()', '.route(PATH, get(handler))', '.route("/x", routes)', '.nest("/v1", other)', '.route("/x", get(handler)']) {
    assert.throws(() => routerOperations(input));
  }
});
test('rejects duplicate operations instead of hiding conflicting contracts', () => {
  assert.throws(() => checkContracts(source, [square, square, admin]), /Duplicate OpenAPI/);
  assert.throws(() => routerOperations(source + '.route("/v1/skills/:name", get(other))'), /Duplicate backend/);
});
