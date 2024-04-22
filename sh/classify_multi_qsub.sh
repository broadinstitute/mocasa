#!/bin/bash

run_script=$0
dir=$(pwd)
n_chunks=10

if [[ $SGE_TASK_ID ]]; then
  /humgen/diabetes2/users/oliverr/pub/bin/mocasa classify -f "$dir"/config.toml -x $n_chunks -k "$SGE_TASK_ID" "$@"
else
  qsub -b y -l h_rt=10:00:00 -pe smp 4 -R y -binding linear:4  -cwd -o "$dir" -t 1-$n_chunks "$run_script" "$@"
fi

