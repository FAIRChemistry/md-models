import * as D from 'io-ts/Decoder';
import { isLeft } from "fp-ts/Either";

// Generic validate function
export function validate<T>(codec: D.Decoder<unknown, T>, value: unknown): T {
  const result = codec.decode(value);
  if (isLeft(result)) {
    throw new Error(D.draw(result.left));
  }
  return result.right;
}

// JSON-LD Types
export interface JsonLdContext {
  [key: string]: any;
}

export interface JsonLd {
  '@context'?: JsonLdContext;
  '@id'?: string;
  '@type'?: string;
}

// none Type definitions
/**
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
    eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
    enim ad minim veniam, quis nostrud exercitation ullamco laboris
    nisi ut aliquip ex ea commodo consequat.

    * @param name - The name of the test. This is a unique identifier that helps track
             individual test cases across the system. It should be
             descriptive and follow the standard naming conventions.
    * @param number
    * @param test2
    * @param ontology
**/
export interface Test extends JsonLd {
  name: string;
  number?: number | null;
  test2?: Test2[] | null;
  ontology?: Ontology | null;
}

export const TestCodec = D.lazy("Test", () => D.struct({
    name: D.string,
    number: D.nullable(D.number),
    test2: D.array(Test2Codec),
    ontology: D.nullable(OntologyCodec),
}));


/**
    * @param names
    * @param number
**/
export interface Test2 extends JsonLd {
  names?: string[] | null;
  number?: number | null;
}

export const Test2Codec = D.lazy("Test2", () => D.struct({
    names: D.array(D.string),
    number: D.nullable(D.number),
}));


// none Enum definitions
export enum Ontology {
  ECO = 'https://www.evidenceontology.org/term/',
  GO = 'https://amigo.geneontology.org/amigo/term/',
  SIO = 'http://semanticscience.org/resource/',
}

export const OntologyCodec = D.union(
  D.literal(Ontology.ECO),
  D.literal(Ontology.GO),
  D.literal(Ontology.SIO),
);