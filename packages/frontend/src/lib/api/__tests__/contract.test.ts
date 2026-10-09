import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

/**
 * Guards the hand-written client types against the checked-in OpenAPI artifact.
 *
 * The frontend types are not generated, so nothing else catches drift between
 * the published contract and the TypeScript interfaces the app relies on. Each
 * mapping below asserts that the interface fields match the schema properties
 * and that every required schema field is non-optional in TypeScript.
 */
const SPEC_PATH = resolve(process.cwd(), '../../docs/api/api-spec.json');
const TYPES_PATH = resolve(process.cwd(), 'src/lib/api/types.ts');

const MAPPINGS: Record<string, string> = {
  AnalysisJob: 'AnalysisJobRead',
  SignalWindowResponse: 'SignalWindowRead',
  SignalChannel: 'SignalChannelRead',
};

interface ParsedInterface {
  fields: Set<string>;
  required: Set<string>;
}

const parseInterfaces = (source: string): Map<string, ParsedInterface> => {
  const interfaces = new Map<string, ParsedInterface>();
  const blockPattern = /export interface (\w+) \{([\s\S]*?)\n\}/g;
  for (const match of source.matchAll(blockPattern)) {
    const [, name, body] = match;
    const fields = new Set<string>();
    const required = new Set<string>();
    for (const line of body.split('\n')) {
      const field = /^ {2}(\w+)(\?)?:/.exec(line);
      if (field) {
        fields.add(field[1]);
        if (!field[2]) {
          required.add(field[1]);
        }
      }
    }
    interfaces.set(name, { fields, required });
  }
  return interfaces;
};

describe('client type contract', () => {
  const spec = JSON.parse(readFileSync(SPEC_PATH, 'utf8')) as {
    components: {
      schemas: Record<string, { properties: Record<string, unknown>; required?: string[] }>;
    };
  };
  const interfaces = parseInterfaces(readFileSync(TYPES_PATH, 'utf8'));

  it.each(Object.entries(MAPPINGS))('%s matches OpenAPI schema %s', (interfaceName, schemaName) => {
    const parsed = interfaces.get(interfaceName);
    expect(parsed, `interface ${interfaceName} not found`).toBeDefined();
    const schema = spec.components.schemas[schemaName];
    expect(schema, `schema ${schemaName} not found`).toBeDefined();

    expect([...(parsed?.fields ?? [])].sort()).toEqual(Object.keys(schema.properties).sort());
    for (const required of schema.required ?? []) {
      expect(parsed?.required.has(required), `${interfaceName}.${required} must be required`).toBe(
        true,
      );
    }
  });
});
