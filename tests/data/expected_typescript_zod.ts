/**
 * This file contains Zod schema definitions for data validation.
 *
 * Zod is a TypeScript-first schema declaration and validation library.
 * It allows you to create schemas that validate data at runtime while
 * providing static type inference.
 *
 * Usage example:
 * ```typescript
 * import { TestSchema } from './schemas';
 *
 * // Validates data at runtime
 * const result = TestSchema.parse(data);
 *
 * // Type-safe - result has correct TypeScript types
 * console.log(result.name);
 *
 * // Will throw error if validation fails
 * try {
 *   TestSchema.parse(invalidData);
 * } catch (err) {
 *   console.error(err);
 * }
 * ```
 *
 * @see https://github.com/colinhacks/zod
 *
 * WARNING: This is an auto-generated file.
 * Do not edit directly - any changes will be overwritten.
 */


import { z } from 'zod';

// JSON-LD Types
export const JsonLdContextSchema = z.record(z.any());

export const JsonLdSchema = z.object({
  '@context': JsonLdContextSchema.optional(),
  '@id': z.string().optional(),
  '@type': z.string().optional(),
});

// Model Type definitions
// Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
// eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim
// ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
// aliquip ex ea commodo consequat.
export const TestSchema = z.lazy(() => JsonLdSchema.extend({
  name: z.string().describe(`
    The name of the test. This is a unique identifier that helps track
    individual test cases across the system. It should be descriptive
    and follow the standard naming conventions.
  `),
  number: z.union([z.number(), z.string()]).nullable(),
  test2: z.array(Test2Schema),
  ontology: OntologySchema.nullable(),
}));

export type Test = z.infer<typeof TestSchema>;

export const Test2Schema = z.lazy(() => JsonLdSchema.extend({
  names: z.array(z.string()),
  number: z.number().nullable(),
}));

export type Test2 = z.infer<typeof Test2Schema>;

// Model Enum definitions
export enum Ontology {
  ECO = 'https://www.evidenceontology.org/term/',
  GO = 'https://amigo.geneontology.org/amigo/term/',
  SIO = 'http://semanticscience.org/resource/',
}

export const OntologySchema = z.nativeEnum(Ontology);