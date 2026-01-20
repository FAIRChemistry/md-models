# SI Unit

This page provides comprehensive information about the structure and components of the data model, including detailed descriptions of the types and their properties, information on enumerations, and an overview of the ontologies used and their associated prefixes. Below, you will find a graph that visually represents the overall structure of the data model.

??? quote "Graph"
    ``` mermaid
    flowchart TB
        unitdefinition(UnitDefinition)
        baseunit(BaseUnit)
        unittype(UnitType)
        unitdefinition(UnitDefinition) --> baseunit(BaseUnit)
        baseunit(BaseUnit) --> unittype(UnitType)

        click unitdefinition "#unitdefinition" "Go to UnitDefinition"
        click baseunit "#baseunit" "Go to BaseUnit"
        click unittype "#unittype" "Go to UnitType"
    ```


## Ontologies
- [qudt](http://qudt.org/schema/qudt#/)
- [om](http://www.ontology-of-units-of-measure.org/resource/om-2/)
- [rdfs](http://www.w3.org/2000/01/rdf-schema#/)
- [unit](http://qudt.org/vocab/unit#/)
- [schema](http://schema.org/)
- [xsd](http://www.w3.org/2001/XMLSchema#/)


## Types


### UnitDefinition
Represents a unit definition that is based on the SI unit system. A unit may be a base unit or a derived unit composed of one or more factor units.

__id__ `string`

- Unique identifier of the unit definition.


__name__ `string`

- Common or human-readable name of the unit definition.


__base_units__ [`list[BaseUnit]`](#baseunit)

- Base or factor units that define this unit.


------

### BaseUnit
Represents a base or factor unit contributing to a derived unit definition.

__kind__* [`UnitType`](#unittype)

- Kind of the base unit (e.g., meter, kilogram, second).


__exponent__* `integer`

- Exponent of the base unit in the unit definition.


__multiplier__ `float`

- Multiplier applied to the base unit.


__scale__ `float`

- Power-of-ten scale applied to the base unit.


## Enumerations

### UnitType

| Alias | Value |
|-------|-------|
| `AMPERE` | ampere |
| `AVOGADRO` | avogadro |
| `BECQUEREL` | becquerel |
| `CANDELA` | candela |
| `CELSIUS` | celsius |
| `COULOMB` | coulomb |
| `DIMENSIONLESS` | dimensionless |
| `FARAD` | farad |
| `GRAM` | gram |
| `GRAY` | gray |
| `HENRY` | henry |
| `HERTZ` | hertz |
| `ITEM` | item |
| `JOULE` | joule |
| `KATAL` | katal |
| `KELVIN` | kelvin |
| `KILOGRAM` | kilogram |
| `LITRE` | litre |
| `LUMEN` | lumen |
| `LUX` | lux |
| `METRE` | metre |
| `MOLE` | mole |
| `NEWTON` | newton |
| `OHM` | ohm |
| `PASCAL` | pascal |
| `RADIAN` | radian |
| `SECOND` | second |
| `SIEMENS` | siemens |
| `SIEVERT` | sievert |
| `STERADIAN` | steradian |
| `TESLA` | tesla |
| `VOLT` | volt |
| `WATT` | watt |
| `WEBER` | weber |