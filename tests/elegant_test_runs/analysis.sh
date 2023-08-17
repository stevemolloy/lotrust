#!/bin/bash
set -x

# Delete outputs and figures
rm -rf ./output
rm ./figures/*
mkdir output

# Run elegant
elegant *.ele

## outputs first-order matrix elements as csv
#sddsprintout -spreadsheet=csv -col={R??} ./output/*_O1.mat ./output/matrix_O1.csv

# Defines z and delta for output file at start of lattice
sddsprocess ./output/w-init.sdds -define=col,z,"-299792458.0 dt * " -process=p,average,pAvg -define=col,delta,"p pAvg - pAvg / " -summarize -noWarnings
sddsprintout -spreadsheet=csv -col={x,xp,y,yp,t,dt,p,z} ./output/w-init.sdds ./output/initial_dist.csv

# Defines z and delta for output file at end of lattice
sddsprocess ./output/w-end.sdds -define=col,z,"-299792458.0 dt * " -process=p,average,pAvg -define=col,delta,"p pAvg - pAvg / " -summarize -noWarnings
sddsprintout -spreadsheet=csv -col={x,xp,y,yp,t,dt,p,z} ./output/w-end.sdds ./output/final_dist.csv

# Python plots
# python3 analysis.py
