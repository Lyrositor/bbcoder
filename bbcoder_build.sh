#!/usr/bin/env bash
export PATH=$(pwd):$PATH
export project=$1
if [ -z "$project" ]; then
    export project="project.xml"
fi
echo "Running bbcoder on $project..."
bbcoder -p "$project" $2
echo "Complete."
echo "Press any key to continue..."
read -n 1
