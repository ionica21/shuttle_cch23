#!/bin/bash

# Array of numbers
numbers=(-1 1 4 5 6 7 8 11 12 13 14 15 18 19 20 21 22)

# Iterate over the numbers and run cch23-validator
for number in "${numbers[@]}"; do
    echo "Running cch23-validator for number: $number"
    cch23-validator "$number"
done
