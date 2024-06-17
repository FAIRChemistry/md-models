---
repo: "https://github.com/JR-1991/md-models"
---

# Equation

The `Equation` type is used to define an equation that can be used in a data model. The equation type includes the equation itself and a list of variables and parameters that are used in the equation.

### Equation

Represents an equation that can be used in a data model.

- __equation__
  - Type: string
  - Description: The equation that is used in the data model.
  - XML: equation
- variables
  - Type: EqVariable[]
  - Description: List of variables that are used in the equation.
  - XML: list_of_variables
- parameters
  - Type: EqParameter[]
  - Description: List of parameters that are used in the equation.
  - XML: list_of_parameters

### EqVariable

Represents a variable that is used in the equation.

- __id__
  - Type: Identifier
  - Description: Unique identifier for the variable.
  - XML: @id
- __name__
  - Type: string
  - Description: Name of the variable.
  - XML: @name
- symbol
  - Type: string
  - Description: Symbol of the variable.
  - XML: @symbol

### EqParameter

Represents a parameter that is used in the equation.

- __id__
  - Type: Identifier
  - Description: Unique identifier for the parameter.
  - XML: @id
- __name__
  - Type: string
  - Description: Name of the parameter.
  - XML: @name
- symbol
  - Type: string
  - Description: Symbol of the parameter.
  - XML: @symbol
- value
  - Type: float
  - Description: Value of the parameter.
  - XML: @value
