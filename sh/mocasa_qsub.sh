#!/bin/bash

run_script=$0

case $1 in
  train)
    action="train"
    ;;
  classify)
    action="classify"
    ;;
  "")
    echo "Need to provide subcommand 'train' or 'classify'."
    exit 1
    ;;
  *)
    echo "Unknown subcommand $1. Valid subcommands are 'train' and 'classify'."
esac