#!/bin/bash

# Grep all *.toml files in the types directory recursively
GEN_FILES=$(find types -name '*.toml')

# Iterate over each file
for file in $GEN_FILES; do
  # Generate the go file
  echo "Generating $file"
  md-models pipeline -i $file
done
