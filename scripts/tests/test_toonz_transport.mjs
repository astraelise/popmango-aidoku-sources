import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
const transport = readFileSync(new URL('../../templates/toonz/src/transport.rs', import.meta.url), 'utf8');
const script = transport.match(/r#"(\(config\) => \{[\s\S]*?)"#/)[1];
function start(rsc = false, cookie = null) {
 const context = { window: {}, document: { cookie: 'existing=kept' }, XMLHttpRequest: class {
  headers = {};
  open(...args) { this.args = args; }
  setRequestHeader(k,v) { this.headers[k] = v; }
  send() { this.sent = true; }
 }};
 const config = { url: 'https://toonz.to/comics?search=a%22b&page=2', cookie, rsc };
 vm.runInNewContext(`(${script})(${JSON.stringify(config)})`, context);
 return { ...context, request: context.window.toonzRequest, config };
}
test('browse preserves URL, applies preferences, and starts a bounded asynchronous request', () => {
 const { request, document, window, config } = start(false, 'reader_prefs=encoded; Path=/; Secure');
 assert.deepEqual(request.args, ['GET', config.url, true]);
 assert.equal(request.timeout, 15000);
 assert.equal(document.cookie, config.cookie);
 assert.equal(request.sent, true);
 assert.equal(window.toonzResponse, null);
 request.status = 200; request.responseText = '<html>streamed content</html>'; request.onload();
 assert.deepEqual(Array.from(window.toonzResponse), [200, request.responseText]);
});
test('reader sets RSC and leaves existing cookies alone', () => {
 const { request, document, window } = start(true);
 assert.equal(request.headers.RSC, '1');
 assert.equal(document.cookie, 'existing=kept');
 request.status = 404; request.responseText = 'missing'; request.onload();
 assert.equal(window.toonzResponse[0], 404);
});
for (const [event, message] of [['onerror','Network request failed'], ['ontimeout','Request timed out'], ['onabort','Request cancelled']]) {
 test(event + ' completes polling with a specific error', () => {
  const { request, window } = start();
  request[event]();
  assert.deepEqual(Array.from(window.toonzResponse), [0, message]);
 });
}

const navigationTemplate = transport.match(/r#"(\(\(\) => \{\{[\s\S]*?)"#/)[1];
function navigate(readyState, marker, reader = false) {
 const selector = reader ? "img[src*='/uploads/chapters/']" : '[data-comic-id]';
 const script = navigationTemplate.replace('{}', JSON.stringify(selector)).replaceAll('{{', '{').replaceAll('}}', '}');
 const context = {
  document: { readyState, querySelector: query => query === selector && marker,
   documentElement: { outerHTML: '<html>complete series or reader</html>' } },
  location: { href: 'https://toonz.to/manhwa/example' },
 };
 return vm.runInNewContext(script, context);
}
test('navigation does not return a refused page without its series marker', () => {
 assert.equal(navigate('complete', false), '');
});
test('navigation waits until the document is parsed', () => {
 assert.equal(navigate('loading', true), '');
});
test('navigation preserves the final redirected URL with its HTML', () => {
 assert.deepEqual(JSON.parse(navigate('interactive', true)),
  ['https://toonz.to/manhwa/example', '<html>complete series or reader</html>']);
});
test('reader navigation requires a chapter image marker', () => {
 assert.equal(navigate('complete', false, true), '');
 assert.equal(JSON.parse(navigate('interactive', true, true))[0], 'https://toonz.to/manhwa/example');
});
