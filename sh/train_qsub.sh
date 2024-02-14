#!/bin/bash

dir=$(pwd)

qsub -b y -l h_rt=4:00:00 -pe smp 4 -R y -binding linear:4  -o "$dir" \
    /humgen/diabetes2/users/oliverr/pub/bin/mocasa train -f "$dir"/config.toml  \
