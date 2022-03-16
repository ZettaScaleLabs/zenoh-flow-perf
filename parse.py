#!/usr/bin/env python3

from io import StringIO
import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
import os
import pandas as pd
from pathlib import Path
import seaborn as sns
import sys
import argparse


palette = {
    'zenoh-flow': 'tab:blue',
    'ros': 'tab:green',
    'ros2': 'tab:orange',
}

# palette = 'bright' #sns.color_palette("bright", 6) #'plasma'
IMG_DIR = Path('img')

def pairwise(data):
    l = iter(data)
    return zip(l,l)


def bytes_label(n):
    kmod = pow(2, 10)
    kdiv = n / kmod
    if kdiv < 1:
        return "{}".format(n)

    mmod = pow(2, 20)
    mdiv = n / mmod
    if mdiv < 1:
        return "{0:.{c}f} KiB".format(kdiv, c=0 if n % kmod == 0 else 2)

    gmod = pow(2, 30)
    gdiv = n / gmod
    if gdiv < 1:
        return "{0:.{c}f} MiB".format(mdiv, c=0 if n % mmod == 0 else 2)

    tmod = pow(2, 40)
    tdiv = n / tmod
    if tdiv < 1:
        return "{0:.{c}f} GiB".format(gdiv, c=0 if n % gmod == 0 else 2)

    pmod = pow(2, 50)
    pdiv = n / pmod
    if pdiv < 1:
        return "{0:.{c}f} TiB".format(tdiv, c=0 if n % tmod == 0 else 2)

    emod = pow(2, 60)
    ediv = n / emod
    if ediv < 1:
        return "{0:.{c}f} PiB".format(pdiv, c=0 if n % pmod == 0 else 2)

    zmod = pow(2, 70)
    zdiv = n / zmod
    if zdiv < 1:
        return "{0:.{c}f} EiB".format(ediv, c=0 if n % emod == 0 else 2)

    ymod = pow(2, 80)
    ydiv = n / ymod
    if ydiv < 1:
        return "{0:.{c}f} ZiB".format(ediv, c=0 if n % zmod == 0 else 2)

    return "{0:.{c}f} YiB".format(ydiv, c=0 if n % ymod == 0 else 2)

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
                log = pd.concat([log,pd.read_csv(l)])
    return log

def mask_first_and_last(x):
    mask = [True]*len(x)
    mask[0] = False
    mask[1] = False
    mask[-2] = False
    mask[-1] = False
    return mask


def prepare(log_dir, kind):
    log = read_log(log_dir)

    # filtering by kind of test
    log = log[log['test']==kind]

    # log['value'] = log['value'].astype(int, errors='ignore')
    log['value'] = pd.to_numeric(log['value'], errors='coerce')

    if kind == 'latency':
        # Remove first and last two samples of every test
        mask = log.groupby(['framework', 'scenario','test','pipeline','payload', 'rate']).transform(
        mask_first_and_last)['value']
    elif kind == 'throughput':
        # Remove first and last two samples of every test
        mask = log.groupby(['framework', 'scenario','test','pipeline','payload']).transform(
        mask_first_and_last)['value']
    log = log.loc[mask]

    if kind == 'latency':
        # this converts everything to seconds, data is expected as micro seconds
        log['value']= [ v/1000000 for v in log['value']] #TODO there is a unit field that should be used for the conversion
        log['label'] = [interval_label(v) for k, v in log['rate'].iteritems()]
        log.sort_values(by='rate', inplace=True)
    elif kind == 'throughput':
        log['label'] = [bytes_label(v) for k, v in log['payload'].iteritems()]
        log.sort_values(by='payload', inplace=True)


    log['framework'] = log['framework'].astype(str)

    # adding count of samples
    log['counter'] = log.groupby(['framework', 'scenario','test','pipeline','payload','rate'])['framework'].cumcount().add(1)

    log = log.reset_index()
    return log

def filter(log, process, msgs=None, pipeline=None):
    layers = log['framework'].unique()

    # filtering if multi process or same process\
    log = log[log['scenario'].isin([process])]



    # filtering is msg/s is set
    if msgs is not None:
        log = log[log['rate']==msgs]

    if pipeline is not None:
        log = log[log['pipeline']==pipeline]

    return log

def resample(log, downsampled_interval=1):
    sequence_interval = 1
    step_size = np.round(downsampled_interval / sequence_interval).astype("int")
    log = log.iloc[::step_size, :]
    log = log.reset_index()
    return log

def compute_overhead(log, overhead):
    couples = []
    for (x,y) in pairwise(overhead):
        couples.append((x,y))

    # overhead computes only with pipeline size equal to 1 (src->op->snk)
    log=log[log['pipeline']==1]

    log_overhead = pd.DataFrame(columns=log.columns)
    i = 0
    dfs = {'framework':[],'scenario':[],'test':[],'pipeline':[],'payload':[],'rate':[],'value':[],'unit':[]}
    for (x,y) in couples:

        # layer,scenario,test,name,messages,pipeline,latency,unit
        msgs = log['rate'].unique()

        log_y = log[log['framework']==y]
        log_x = log[log['framework']==x]

        for m in msgs:
            df_x = log_x[log_x['rate']==m]

            median_latency_x = np.median(df_x['value'])*2

            df_y = log_y[log_y['rate']==m]

            median_latency_y = np.median(df_y['value'])

            latency_diff = median_latency_y - median_latency_x

            dfs['framework'].append(f'{y}-overhead')
            dfs['scenario'].append('multi')
            dfs['test'].append('latency')
            dfs['pipeline'].append(1)
            dfs['payload'].append(8)
            dfs['rate'].append(m)
            dfs['unit'].append('us')
            i+=1

    log_overhead = pd.concat([log_overhead, pd.DataFrame.from_dict(dfs)])

    return log_overhead

def latency_ecfd_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    g = sns.ecdfplot(data=log, x='value', palette=palette, hue='framework', label='framework')


    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    if scale == 'log':
        g.set_xscale('log')

    plt.xticks(rotation=72.5)
    plt.xlabel('Latency (seconds)')

    # plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    # ticker = mpl.ticker.EngFormatter(unit='')
    # axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def latency_pdf_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    g = sns.displot(data=log, x='value', palette=palette, hue='framework', label='framework')


    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    # if scale == 'log':
    #     g.set_xscale('log')

    plt.xticks(rotation=72.5)
    plt.xlabel('Latency (seconds)')

    # plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    # ticker = mpl.ticker.EngFormatter(unit='')
    # axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def latency_stat_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    g = sns.lineplot(data=log, x='label', y='value', palette=palette,
                ci=95, err_style='band', hue='framework',
                estimator=np.median, style='pipeline')

    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5)
    plt.xlabel('Messages per seconds (msg/s)')

    plt.ylabel('Latency (seconds)')
    plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def throughput_stat_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    g = sns.lineplot(data=log, x='label', y='value', palette=palette,
                ci=95, err_style='band', hue='framework',
                estimator=np.median, style='pipeline')

    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5)
    plt.xlabel('Payload size (bytes)')

    plt.ylabel('Messages per second (msg/s)')
    plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))

def latency_time_plot(log, scale, outfile):

    # set same samples size for all layers
    layers = log['framework'].unique()
    maxs = []
    for l in layers:
        maxs.append(log[log['framework']==l]['counter'].max())

    min_max = min(maxs)

    log = log[log['counter']<=min_max]

    fig, axes = plt.subplots()

    g = sns.lineplot(data=log, x='counter',y='valye', palette=palette,
                #ci='sd', err_style='band', estimator="median",
                hue='framework')#, style='pipeline')

    if scale == 'log':
        g.set_yscale('log')


    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5)
    plt.xlabel('Index')

    plt.ylabel('Latency (seconds)')
    plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))


    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()

    fig.savefig(IMG_DIR.joinpath(outfile))




def main():
    parser = argparse.ArgumentParser(description='Parse zenoh flow performance results')
    parser.add_argument('-k','--kind', help='Kind of the tests', required=False, choices=['latency', 'throughput'], default='latency')
    parser.add_argument('-d','--data', help='Logs directory', required=True, type=str)
    parser.add_argument('-p','--process', help='Single process or multi process', choices=['single', 'multi', 'all'], default='single', required=False)
    parser.add_argument('-t','--type', help='Plot type', choices=['stat', 'time', 'ecdf', 'pdf'], default='stat', required=False)
    parser.add_argument('-s','--scale', help='Plot scale', choices=['log', 'lin'], default='log', required=False)
    parser.add_argument('-m','--msgs', help='Filter for this # of msg/s', required=False, type=int)
    parser.add_argument('-l','--length', help='Filter for this pipeline length', required=False, type=int)
    parser.add_argument('-o','--output', help='Output file name', required=False, type=str, default='plot.pdf')
    parser.add_argument('-r','--resample', help='Resample the data', required=False, type=int)
    parser.add_argument('--overhead', help="Compute the overhead", required=False, nargs='+', default=[])

    args = vars(parser.parse_args())
    data = args['data']
    print(f'[ START ] Processing data in { data }')

    if not os.path.exists(IMG_DIR):
        os.makedirs(IMG_DIR)

    log = prepare(args['data'], args['kind'])
    print(f'[ STEP1 ] Read a total of {log.size} samples')
    log = filter(log, args['process'], args.get('msgs', None), args.get('length', None))
    print(f'[ STEP2 ] After filtering we have {log.size} samples')
    if log.size == 0:
        print(f'[ ERR ] Cannot continue without samples!')
        exit(-1)
    if args['resample'] is not None:
        log = resample(log, args['resample'])
        print(f'[ STEP3 ] After resampling we have {log.size} samples')
        if log.size == 0:
            print(f'[  ERR  ] Cannot continue without samples!')
            exit(-1)

    if len(args['overhead']) > 0 and len(args['overhead']) % 2 == 0:
        log = compute_overhead(log, args['overhead'])
        print(f'[ STEP4 ] After computing overhead we have {log.size} samples')
        if log.size == 0:
            print(f'[  ERR  ] Cannot continue without samples!')
            exit(-1)

    if args['kind'] == 'latency':
        if args['type'] == 'stat':
            latency_stat_plot(log, args['scale'], args['output'])
        elif args['type'] == 'time':
            latency_time_plot(log, args['scale'], args['output'])
        elif args['type'] == 'ecdf':
            latency_ecfd_plot(log, args['scale'], args['output'])
        elif args['type'] == 'pdf':
            latency_pdf_plot(log, args['scale'], args['output'])
    elif args['kind'] == 'throughput':
        if args['type'] == 'stat':
            throughput_stat_plot(log, args['scale'], args['output'])
        elif args['type'] == 'time':
            print('Not implemented.')
            # latency_time_plot(log, args['scale'], args['output'])
        elif args['type'] == 'ecdf':
            print('Not implemented.')
            # latency_ecfd_plot(log, args['scale'], args['output'])
        elif args['type'] == 'pdf':
            print('Not implemented.')
            # latency_pdf_plot(log, args['scale'], args['output'])

    out = IMG_DIR.joinpath(args['output'])
    print(f'[  DONE ] File saved to { out }')




if __name__=='__main__':
    main()