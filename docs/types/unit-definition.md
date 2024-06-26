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


## Types


### UnitDefinition
Represents a unit definition that is based on the SI unit system.

__id__ `string`

- Unique identifier of the unit definition.


__name__ `string`

- Common name of the unit definition.


__base_units__ [`list[BaseUnit]`](#baseunit)

- Base units that define the unit.


------

### BaseUnit
Represents a base unit in the unit definition.

__kind__* [`UnitType`](#unittype)

- Kind of the base unit (e.g., meter, kilogram, second).


__exponent__* `integer`

- Exponent of the base unit in the unit definition.


__multiplier__ `float`

- Multiplier of the base unit in the unit definition.


__scale__ `float`

- Scale of the base unit in the unit definition.


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