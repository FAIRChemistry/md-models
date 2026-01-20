---
repo: "https://fairchemistry.org/md-models/"
prefix: "mdmodels"
prefixes:
  qudt: "http://qudt.org/schema/qudt#"
  unit: "http://qudt.org/vocab/unit#"
  om: "http://www.ontology-of-units-of-measure.org/resource/om-2/"
  rdfs: "http://www.w3.org/2000/01/rdf-schema#"
  xsd: "http://www.w3.org/2001/XMLSchema#"
  schema: "http://schema.org/"
---

# SI Unit

This data model implements a set of SI units for the representation of physical quantities.
The model is aligned with the International System of Units (SI) and semantically compatible
with SBML unit definitions by reusing established Semantic Web vocabularies.

> This is a direct semantic implementation of the SBML UnitDefinition concept.

### UnitDefinition (qudt:Unit)

Represents a unit definition that is based on the SI unit system.
A unit may be a base unit or a derived unit composed of one or more factor units.

- id
  - Type: string
  - Description: Unique identifier of the unit definition.
  - Term: schema:identifier
  - XML: @id
- name
  - Type: string
  - Description: Common or human-readable name of the unit definition.
  - Term: rdfs:label
  - XML: @name
- base_units
  - Type: BaseUnit[]
  - Description: Base or factor units that define this unit.
  - Term: qudt:factorUnit

### BaseUnit (qudt:FactorUnit)

Represents a base or factor unit contributing to a derived unit definition.

- __kind__
  - Type: UnitType
  - Description: Kind of the base unit (e.g., meter, kilogram, second).
  - Term: qudt:unit
  - XML: @kind
- __exponent__
  - Type: integer
  - Description: Exponent of the base unit in the unit definition.
  - Term: qudt:exponent
  - XML: @exponent
- multiplier
  - Type: float
  - Description: Multiplier applied to the base unit.
  - Term: qudt:multiplier
  - XML: @multiplier
- scale
  - Type: float
  - Description: Power-of-ten scale applied to the base unit.
  - Term: qudt:scale
  - XML: @scale

## Enumerations

### UnitType

Enumeration of standard SI and derived unit types used in scientific measurements and calculations.

```
AMPERE = ampere
FARAD = farad
JOULE = joule
LUX = lux
RADIAN = radian
VOLT = volt
AVOGADRO = avogadro
GRAM = gram
KATAL = katal
METRE = metre
SECOND = second
WATT = watt
BECQUEREL = becquerel
GRAY = gray
KELVIN = kelvin
CELSIUS = celsius
MOLE = mole
SIEMENS = siemens
WEBER = weber
CANDELA = candela
HENRY = henry
KILOGRAM = kilogram
NEWTON = newton
SIEVERT = sievert
COULOMB = coulomb
HERTZ = hertz
LITRE = litre
OHM = ohm
STERADIAN = steradian
DIMENSIONLESS = dimensionless
ITEM = item
LUMEN = lumen
PASCAL = pascal
TESLA = tesla
```
