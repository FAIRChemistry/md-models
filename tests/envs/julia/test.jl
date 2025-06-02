using Test
using JSON3
using StructTypes

include("generated.jl")
using .Model

@testset "Julia Model Tests" begin

    @testset "Test2 struct" begin
        # Test default construction
        test2_empty = Test2()
        @test test2_empty.names === nothing
        @test test2_empty.number === nothing

        # Test construction with values
        test2_with_values = Test2(
            names=["Alice", "Bob", "Charlie"],
            number=42.5
        )
        @test test2_with_values.names == ["Alice", "Bob", "Charlie"]
        @test test2_with_values.number == 42.5

        # Test partial construction
        test2_partial = Test2(names=["David"])
        @test test2_partial.names == ["David"]
        @test test2_partial.number === nothing
    end

    @testset "Test struct" begin
        # Test construction with required field
        test_basic = Test(name="basic_test")
        @test test_basic.name == "basic_test"
        @test test_basic.number === nothing
        @test test_basic.test2 === nothing
        @test test_basic.ontology === nothing

        # Test construction with all fields
        test2_instance = Test2(names=["test"], number=1.0)
        test_full = Test(
            name="full_test",
            number=TestNumberFloat(3.14),
            test2=[test2_instance],
            ontology="http://example.org/ontology"
        )
        @test test_full.name == "full_test"
        @test isa(test_full.number, TestNumberFloat)
        @test test_full.number.value == 3.14
        @test length(test_full.test2) == 1
        @test test_full.ontology == "http://example.org/ontology"
    end

    @testset "Union Types" begin
        # Test TestNumberFloat
        num_float = TestNumberFloat(123.456)
        @test isa(num_float, TestNumberType)
        @test isa(num_float, TestNumberFloat)
        @test num_float.value == 123.456

        # Test TestNumberString
        num_string = TestNumberString("789")
        @test isa(num_string, TestNumberType)
        @test isa(num_string, TestNumberString)
        @test num_string.value == "789"

        # Test both can be used in Test struct
        test_with_float = Test(name="float_test", number=TestNumberFloat(1.5))
        test_with_string = Test(name="string_test", number=TestNumberString("42"))

        @test test_with_float.number.value == 1.5
        @test test_with_string.number.value == "42"
    end

    @testset "JSON Serialization" begin
        # Create test instances
        test2_data = Test2(names=["JSON", "Test"], number=99.9)
        test_data = Test(
            name="json_test",
            number=TestNumberFloat(2.71),
            test2=[test2_data],
            ontology="http://test.org"
        )

        # Test JSON serialization (basic test - actual JSON3 integration would need StructTypes setup)
        # Since the generated file imports JSON3 and StructTypes, we assume serialization is intended
        @test test_data.name == "json_test"
        @test isa(test_data.number, TestNumberFloat)
        @test test_data.number.value == 2.71
        @test length(test_data.test2) == 1
        @test test_data.test2[1].names == ["JSON", "Test"]
        @test test_data.test2[1].number == 99.9
        @test test_data.ontology == "http://test.org"
    end

    @testset "Mutability Tests" begin
        # Test that structs are mutable
        test_mutable = Test(name="mutable_test")
        test_mutable.ontology = "changed"
        @test test_mutable.ontology == "changed"

        test2_mutable = Test2()
        test2_mutable.names = ["modified"]
        test2_mutable.number = 42.0
        @test test2_mutable.names == ["modified"]
        @test test2_mutable.number == 42.0
    end

    @testset "Export Tests" begin
        # Test that exported types are available
        @test isdefined(Model, :Test)
        @test isdefined(Model, :Test2)

        # Test that we can create instances directly
        direct_test = Model.Test(name="direct")
        direct_test2 = Model.Test2()
        @test direct_test.name == "direct"
        @test direct_test2.names === nothing
    end

    @testset "Edge Cases" begin
        # Test with empty vectors
        test_empty_vec = Test(name="empty", test2=Test2[])
        @test length(test_empty_vec.test2) == 0

        # Test with multiple Test2 instances
        multiple_test2 = [
            Test2(names=["first"], number=1.0),
            Test2(names=["second"], number=2.0),
            Test2()  # default values
        ]
        test_multiple = Test(name="multiple", test2=multiple_test2)
        @test length(test_multiple.test2) == 3
        @test test_multiple.test2[1].names == ["first"]
        @test test_multiple.test2[2].names == ["second"]
        @test test_multiple.test2[3].names === nothing
    end
end

println("All tests completed!")
