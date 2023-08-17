# %%
import numpy as np
import matplotlib.pyplot as plot
import csv
import pandas as pd

plot.rcParams.update({'font.size':22, 'figure.figsize':(12,7), 'figure.max_open_warning':0, 'figure.facecolor':'white'})
msize = 10

Q = 100e-12
bins = 400
t_lim = 2000


##
initial_dist = pd.read_csv('./output/initial_dist.csv')

initial_tavg = np.mean(initial_dist['dt'])
initial_pavg = np.mean(initial_dist['p'])

initial_t_hist = (initial_dist['dt']-initial_tavg)*1e15
initial_p_hist = (initial_dist['p']*0.511)/1000

initial_t_count = np.histogram(initial_t_hist, bins=bins, density=True)
initial_p_count = np.histogram(initial_p_hist, bins=bins, density=True)

##
final_dist_O1 = pd.read_csv('./output/final_dist.csv')

final_tavg_O1 = np.mean(final_dist_O1['dt'])
final_pavg_O1 = np.mean(final_dist_O1['p'])

final_t_hist_O1 = (final_dist_O1['dt']-final_tavg_O1)*1e15
final_p_hist_O1 = (final_dist_O1['p']*0.511)/1000

final_t_count_O1 = np.histogram(final_t_hist_O1, bins=bins, density=True)
final_p_count_O1 = np.histogram(final_p_hist_O1, bins=bins, density=True)



plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r't [fs]')
ax1.errorbar((initial_dist['dt']-initial_tavg)*1e15, (initial_dist['p']*0.511)/1000, fmt='b.',markersize=msize, zorder=1, label=r"Initial")
ax1.errorbar((final_dist_O1['dt']-final_tavg_O1)*1e15, (final_dist_O1['p']*0.511)/1000, fmt='r.',markersize=msize, zorder=2, label=r"$1^{st}$-order")
ax1.set_ylabel(r"Energy [GeV]")
plot.legend()
plot.savefig('./figures/t-p_norm_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)

plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r'z [$\mu m$]')
ax1.errorbar(-(initial_dist['dt']-initial_tavg)*(3e8)*(1e6), (1e2)*(initial_dist['p']-initial_pavg)/initial_pavg, fmt='b.',markersize=msize, zorder=1, label=r"Initial")
ax1.errorbar(-(final_dist_O1['dt']-final_tavg_O1)*(3e8)*(1e6), (1e2)*(final_dist_O1['p']-final_pavg_O1)/final_pavg_O1, fmt='r.',markersize=msize, zorder=2, label=r"$1^{st}$-order")
ax1.set_ylabel(r"$\delta$ [%]")
plot.legend()
plot.savefig('./figures/z-delta_norm_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)

plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r't [fs]')
ax1.errorbar((final_dist_O1['dt'])*1e15, (final_dist_O1['p']*0.511)/1000, fmt='r.',markersize=msize, zorder=1, label=r"$1^{st}$-order")
ax1.set_ylabel(r"Energy [GeV]")
plot.legend()
plot.savefig('./figures/t-p_end_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)

plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r'z [$\mu m$]')
ax1.errorbar(-(final_dist_O1['dt'])*(3e8)*(1e6), (1e2)*(final_dist_O1['p']-final_pavg_O1)/final_pavg_O1, fmt='r.',markersize=msize, zorder=1, label=r"$1^{st}$-order")
ax1.set_ylabel(r"$\delta$ [%]")
plot.legend()
plot.savefig('./figures/z-delta_end_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)

plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r'$\Delta$z [$\mu m$]')
ax1.errorbar(-(final_dist_O1['dt']-initial_dist['dt'])*(3e8)*(1e6), (1e2)*((final_dist_O1['p']-final_pavg_O1)/final_pavg_O1), fmt='r.',markersize=msize, zorder=1, label=r"$1^{st}$-order")
ax1.set_ylabel(r"$\delta$ [%]")
plot.legend()
plot.savefig('./figures/delz-delta_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)

plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r't [fs]')
ax1.errorbar((initial_dist['dt'])*1e15, (initial_dist['p']*0.511)/1000, fmt='r.',markersize=msize, zorder=1)
ax1.set_ylabel(r"Energy [GeV]")
plot.savefig('./figures/t-p_initial_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)

plot.figure()
fig, ax1 = plot.subplots()
ax1.set_xlabel(r'z [$\mu m$]')
ax1.errorbar(-(initial_dist['dt'])*(3e8)*(1e6), (1e2)*(initial_dist['p']-initial_pavg)/initial_pavg, fmt='r.',markersize=msize, zorder=2)
ax1.set_ylabel(r"$\delta$ [%]")
plot.savefig('./figures/z-delta_initial_py.png', bbox_inches='tight', pad_inches=0.1, dpi=600)




