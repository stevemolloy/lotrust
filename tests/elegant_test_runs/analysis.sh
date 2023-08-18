#!/bin/bash
set -x

ele_list=`tail -n 21 test_lines.lte | awk 'BEGIN { FS = ":" } ; { print $1 }'`

# Clean up anything from a prev run
rm -rf ./output

for ele in $ele_list
do
  mkdir -p output/$ele

  # Run elegant
  elegant test_$ele.ele

  mv output/w-init.sdds output/$ele/
  mv output/w-end.sdds output/$ele/
 
  ## outputs first-order matrix elements as csv
  #sddsprintout -spreadsheet=csv -col={R??} ./output/*_O1.mat ./output/matrix_O1.csv
  
  # Defines some stuff for output file at start of lattice
  sddsprocess ./output/$ele/w-init.sdds \
    -define=col,gamma,"p p * 1 + sqrt" \
    -define=col,beta,"p gamma / " \
    -define=col,z,"beta 299792458 dt -1 * * *" \
    -define=col,ke,"gamma 1 - 510998.949996164 *" \
    -process=ke,average,keAvg \
    -define=col,delta,"ke keAvg -" \
    -summarize \
    -noWarnings
  sddsprintout -spreadsheet=csv -columns ./output/$ele/w-init.sdds ./output/$ele/initial_dist.csv
  
  # Defines some stuff for output file at end of lattice
  sddsprocess ./output/$ele/w-end.sdds \
    -define=col,gamma,"p p * 1 + sqrt" \
    -define=col,beta,"p gamma / " \
    -define=col,z,"beta 299792458 dt -1 * * *" \
    -define=col,ke,"gamma 1 - 510998.949996164 *" \
    -process=ke,average,keAvg \
    -define=col,delta,"ke keAvg -" \
    -summarize \
    -noWarnings
  sddsprintout -spreadsheet=csv -columns ./output/$ele/w-end.sdds ./output/$ele/final_dist.csv
done

# Python plots
# python3 analysis.py
