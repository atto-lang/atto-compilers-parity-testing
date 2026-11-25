// Standalone source-free policy model; NOT an AttoLang compiler test.
// These token vectors contain no source files or Git objects from another org.
import test from 'node:test';
import assert from 'node:assert/strict';

const DECLS = new Set(['fnc', 'routine', 'class', 'actor', 'trait', 'interface', 'contract']);
function normalizePrefix(words, scope) {
  const tokens = words.trim().split(/\s+/);
  const defines = tokens.filter(x => x === 'define').length;
  const publics = tokens.filter(x => x === 'pub').length;
  if (defines > 1 || publics > 1) throw new Error('duplicate declaration modifier');
  const kind = tokens.find(x => DECLS.has(x));
  if (!kind) throw new Error('not a declaration');
  if (scope === 'executable') throw new Error('cannot define within executable body');
  if ((scope === 'class' || scope === 'actor') &&
      ['class', 'actor', 'trait', 'interface', 'contract'].includes(kind))
    throw new Error('nested type forbidden');
  return [scope === 'root' ? 'define' : null, publics ? 'pub' : null, kind]
    .filter(Boolean).join(' ');
}
function admitsFactory(record) {
  if (record.actor || record.abstract || record.genericClass) return false;
  if (record.marker !== 'Newable[Cart]' || record.markerOwner !== 'compiler') return false;
  if (!record.declared || !record.static || !record.public) return false;
  if (record.async || record.suspends || record.methodGenerics || record.parameters !== 0) return false;
  return record.returnType === 'Cart';
}
const validFactory = Object.freeze({
  actor: false, abstract: false, genericClass: false, marker: 'Newable[Cart]',
  markerOwner: 'compiler', declared: true, static: true, public: true,
  async: false, suspends: false, methodGenerics: false, parameters: 0,
  returnType: 'Cart'
});

test('root visibility permutations share one canonical declaration identity', () => {
  for (const head of ['pub define fnc', 'define pub fnc', 'pub fnc']) {
    assert.equal(normalizePrefix(head, 'root'), 'define pub fnc');
  }
});
test('nested members omit define without changing public visibility', () => {
  for (const head of ['pub define fnc', 'define pub fnc', 'pub fnc']) {
    assert.equal(normalizePrefix(head, 'class'), 'pub fnc');
    assert.equal(normalizePrefix(head, 'module'), 'pub fnc');
  }
});
test('duplicate declaration modifiers are never canonicalized away', () => {
  for (const head of ['pub define pub fnc', 'define define pub fnc', 'define pub pub class'])
    assert.throws(() => normalizePrefix(head, 'root'), /duplicate/);
});
test('executable bodies and nested type declarations remain fail-closed', () => {
  assert.throws(() => normalizePrefix('define fnc', 'executable'), /executable/);
  assert.throws(() => normalizePrefix('define class', 'actor'), /nested type/);
  assert.throws(() => normalizePrefix('pub define trait', 'class'), /nested type/);
});
test('user-defined .new has a sealed static Newable witness in reference policy', () => {
  assert.equal(admitsFactory(validFactory), true);
  const invalid = [
    {marker: ''}, {marker: 'Newable[int]'}, {markerOwner: 'user'},
    {actor: true}, {abstract: true}, {genericClass: true},
    {declared: false}, {static: false}, {public: false},
    {async: true}, {suspends: true}, {methodGenerics: true},
    {parameters: 1}, {returnType: 'int'}
  ];
  for (const violation of invalid)
    assert.equal(admitsFactory({...validFactory, ...violation}), false,
      JSON.stringify(violation));
});
