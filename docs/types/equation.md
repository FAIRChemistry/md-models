# Equation

This page provides comprehensive information about the structure and components of the data model, including detailed descriptions of the types and their properties, information on enumerations, and an overview of the ontologies used and their associated prefixes. Below, you will find a graph that visually represents the overall structure of the data model.

??? quote "Graph"
    ``` mermaid
    flowchart TB
        equation(Equation)
        eqvariable(EqVariable)
        eqparameter(EqParameter)
        equation(Equation) --> eqvariable(EqVariable)
        equation(Equation) --> eqparameter(EqParameter)

        click equation "#equation" "Go to Equation"
        click eqvariable "#eqvariable" "Go to EqVariable"
        click eqparameter "#eqparameter" "Go to EqParameter"
    ```


## Types


### Equation
Represents an equation that can be used in a data model.

__equation__* `string`

- The equation that is used in the data model.


__variables__ [`list[EqVariable]`](#eqvariable)

- List of variables that are used in the equation.


__parameters__ [`list[EqParameter]`](#eqparameter)

- List of parameters that are used in the equation.


------

### EqVariable
Represents a variable that is used in the equation.

__id__* `string`

- Unique identifier for the variable.


__name__* `string`

- Name of the variable.


__symbol__ `string`

- Symbol of the variable.


------

### EqParameter
Represents a parameter that is used in the equation.

__id__* `string`

- Unique identifier for the parameter.


__name__* `string`

- Name of the parameter.


__symbol__ `string`

- Symbol of the parameter.


__value__ `float`

- Value of the parameter.