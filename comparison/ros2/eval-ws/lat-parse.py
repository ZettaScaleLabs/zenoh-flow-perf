from io import StringIO
import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
import os
import pandas as pd
from pathlib import Path
import seaborn as sns
import sys

palette = 'plasma'
img_dir = Path('img')

if not os.path.exists(img_dir):
    os.makedirs(img_dir)


def interval_label(n):
    if n == 0:
        return "inf"

    if n == 1:
        return "1"

    if n == 10:
        return "10"

    if n == 100:
        return "100"

    if n == 1000:
        return "1 K"

    if n == 10000:
        return "10 K"

    if n == 100000:
        return "100 K"

    if n == 1000000:
        return "1 M"

    return "1 M"



def read_log(log_dir):
    log = None
    for l in os.scandir(log_dir):
        if l.is_file():
            if log is None:
                log = pd.read_csv(l)
            else:
                log = log.append(pd.read_csv(l))
    return log

def mask_first_and_last(x):
    mask = [True]*len(x)
    mask[0] = False
    mask[1] = False
    mask[-2] = False
    mask[-1] = False
    return mask


log_dir = Path(sys.argv[1])
# Read tests logs
log = read_log(log_dir)

# print(log)
# Remove first and last two samples of every test
mask = log.groupby(['layer', 'test' ,'name','messages','pipeline']).transform(
    mask_first_and_last)['latency']
log = log.loc[mask]
print(log)
print(log['messages'].unique())
log['latency']= [ v/1000000 for v in log['latency']]
log.sort_values(by='messages', inplace=True)
log['name'] = log['name'].astype(str)
log['label'] = [interval_label(v) for k, v in log['messages'].iteritems()]
# log['throughput'] = 8 * log['size'] * log['messages']
log['layer'] = [ f"{key} - {val}" for key, val in zip(log['layer'], log['pipeline'])]

log = log.reset_index()

# ALL msg/s
fig, axes = plt.subplots()

g = sns.lineplot(data=log, x='label', y='latency', estimator="median",
             ci='sd', err_style='band', palette=palette,
             hue='layer')
g.set_yscale('log')

#g.set_xticklabels(log['messages'].unique())
plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

plt.xticks(rotation=72.5)
plt.xlabel('Messages per seconds (msg/s)')

plt.ylabel('Latency (micro-seconds)')
plt.legend(title='Layer')

plt.yticks([pow(10, -5)] + [i*pow(10, -5)  for i in range(2, 21, 2)] + [i*pow(10, -4) for i in range(2, 21, 2)], fontsize=4)

ticker = mpl.ticker.EngFormatter(unit='')
axes.yaxis.set_major_formatter(ticker)

plt.tight_layout()
#plt.show()
fig.savefig(img_dir.joinpath('ros2-latency-all.pdf'))

# ALL throughput
# fig, axes = plt.subplots()

# g = sns.lineplot(data=log, x='label', y='throughput', estimator="median",
#              ci='sd', err_style='band', palette=palette,
#              hue='layer')
# g.set_yscale('log')

# plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
# plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

# plt.xticks(rotation=72.5)
# plt.xlabel('Payload size (Bytes)')

# plt.ylabel('bit/s')
# plt.legend(title='Layer')
# ticker = mpl.ticker.EngFormatter(unit='')
# axes.yaxis.set_major_formatter(ticker)

# plt.tight_layout()
# #plt.show()
# fig.savefig(img_dir.joinpath('zenoh-flow-thr-all.pdf'))
