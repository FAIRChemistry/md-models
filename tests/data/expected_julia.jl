# This file contains Julia type definitions with JSON serialization.
#
# WARNING: This is an auto-generated file.
# Do not edit directly - any changes will be overwritten.

module Model

using JSON3
using StructTypes

#=
  Type Definitions
  ---------------
  Main struct definitions with their fields and JSON serialization support.
=#


Base.@kwdef mutable struct Test2
    names::Union{Vector{ String }, Nothing} = nothing

    number::Union{ Float64, Nothing} = nothing

end

export Test2


#=
  Union Type Definitions for Test.number
  ---------------------
  Custom union types for fields that can accept multiple types.
=#

"""
Union type for Test.number
"""
abstract type TestNumberType end

struct TestNumberFloat <: TestNumberType
    value::Float64
end

struct TestNumberString <: TestNumberType
    value::String
end


"""
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do
eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim
ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
aliquip ex ea commodo consequat.
"""
Base.@kwdef mutable struct Test
    """
    The name of the test. This is a unique identifier that helps track
    individual test cases across the system. It should be descriptive
    and follow the standard naming conventions.
    """
    name::String

    number::Union{ TestNumberType, Nothing} = nothing

    test2::Union{Vector{ Test2 }, Nothing} = nothing

    ontology::Union{ String, Nothing} = nothing

end

export Test


end # module none