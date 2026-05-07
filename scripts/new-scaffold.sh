#!/bin/bash

# the base directory of the project
BASE_DIR=/workspaces/zeitrak/zeitrak-core/src

# get the user defined name
read -p "Enter the scope of the new module: " scope
read -p "Enter the name of the new module: " module_name

# create the module directory
mkdir -p "$BASE_DIR/$scope/$module_name"

# copy the scaffold files
cp -r /workspaces/zeitrak/scripts/resources/new_scaffold/* "$BASE_DIR/$scope/$module_name"
