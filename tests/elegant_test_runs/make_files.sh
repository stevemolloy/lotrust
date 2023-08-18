#!/bin/bash

set -x

ele_list=`tail -n 21 test_lines.lte | awk 'BEGIN { FS = ":" } ; { print $1 }'`

for ele in $ele_list
do
  sed "s/RFCW_CREST/${ele}/g" test_lines.ele.bak > test_$ele.ele
done

