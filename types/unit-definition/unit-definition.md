---
repo: "https://github.com/JR-1991/md-models"
---

# SI Unit

This data model implements a set of SI units for the representation of physical quantities. The SI unit system is based on seven base units, which are used to derive all other units. The base units are given in [UnitType](#UnitType). The unit definitions are based on the International System of Units (SI) and are defined in terms of the base units. The unit definitions are given in [UnitDefinition](#UnitDefinition).

> This is a direct implementation of the [SBML unit definition](https://synonym.caltech.edu/software/libsbml/5.18.0/docs/formatted/python-api/classlibsbml_1_1_unit_definition.html).

### UnitDefinition

Represents a unit definition that is based on the SI unit system.

- id
  - Type: string
  - Description: Unique identifier of the unit definition.
  - XML: @id
- name
  - Type: string
  - Description: Common name of the unit definition.
  - XML: @name
- base_units
  - Type: BaseUnit[]
  - Description: Base units that define the unit.

### BaseUnit

Represents a base unit in the unit definition.

- __kind__
  - Type: UnitType
  - Description: Kind of the base unit (e.g., meter, kilogram, second).
  - XML: @kind
- __exponent__
  - Type: integer
  - Description: Exponent of the base unit in the unit definition.
  - XML: @exponent
- multiplier
  - Type: float
  - Description: Multiplier of the base unit in the unit definition.
  - XML: @multiplier
- scale
  - Type: float
  - Description: Scale of the base unit in the unit definition.
  - XML: @scale

## Enumerations

### UnitType

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
