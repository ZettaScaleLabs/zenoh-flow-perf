#!/usr/bin/env python3
import os
from io import StringIO
import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
import math

# os.environ["MODIN_ENGINE"] = "ray"  # Modin will use Dask

import pandas as pd
# import modin.pandas as pd
from pathlib import Path
import seaborn as sns
import sys
import argparse
# import ray

# ray.init()

palette = {
    'zenoh-flow': 'tab:blue',
    'erdos': 'tab:green',
}

palette_medians = {
    'zenoh-flow': 'red',
    'erdos': 'red',
}


styles = {
    'single': (0,0),
    'multi': (1,1),
}

labels = ['Zenoh Flow','ERDOS']
hue_order = ['zenoh-flow', 'erdos']

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

def convert_value(line):
    if line.unit == 's':
        return line.value
    if line.unit == 'ms':
        return line.value / pow(10,3)
    if line.unit == 'us':
        return line.value / pow(10,6)
    if line.unit == "ns":
        return line.value / pow(10,9)

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

    log['value'] = pd.to_numeric(log['value'], errors='coerce')
    if kind == 'latency':
        # Remove first and last two samples of every test
        mask = log.groupby(['framework', 'scenario','pipeline', 'test','payload', 'rate']).transform(
        mask_first_and_last)['value']
    elif kind == 'throughput':
        None
        # Remove first and last two samples of every test
        # mask = log.groupby(['framework', 'scenario','pipeline','test','payload', 'rate']).transform(
        # mask_first_and_last)['value']
    # log = log.loc[mask]

    if kind == 'latency':
        # this converts everything to seconds, data is expected as micro seconds
        log['value']= log.apply(convert_value, axis=1)
        log = log[log['value']>pow(10,-9)]
        log['label'] = [interval_label(v) for k, v in log['rate'].iteritems()]
        log.sort_values(by='rate', inplace=True, ascending=False)

    elif kind == 'throughput':
        log['label'] = [bytes_label(v) for k, v in log['payload'].iteritems()]
        log.sort_values(by='payload', inplace=True)

    log['framework'] = log['framework'].astype(str)

    log = log.reset_index()

    return log

def filter(log, scenario=None, rate=None):
    layers = log['framework'].unique()

    if scenario is not None:
        # filtering if tcp or udl
        log = log[log['scenario'].isin([scenario])]

    # filtering is msg/s is set
    if rate is not None:
        log = log[log['rate']==rate]

    return log



def rtt_ecfd_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    g = sns.ecdfplot(data=log, x='value', palette=palette, hue='framework', label='framework')


    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    if scale == 'log':
        g.set_xscale('log')

    plt.xticks(rotation=72.5)
    plt.xlabel('RTT (seconds)')

    # plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    # ticker = mpl.ticker.EngFormatter(unit='')
    # axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def rtt_pdf_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    g = sns.displot(data=log, x='value', palette=palette, hue='framework', label='framework')


    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    # if scale == 'log':
    #     g.set_xscale('log')

    plt.xticks(rotation=72.5)
    plt.xlabel('RTT (seconds)')

    # plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    # ticker = mpl.ticker.EngFormatter(unit='')
    # axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def rtt_stat_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    ticks = [1,10,100,1000,10000,100000,1000000,10000000]

    g = sns.lineplot(data=log, x='label', y='value', palette=palette,
                ci=95, err_style='band', hue='framework',
                estimator=np.median, style='scenario', dashes=styles)

    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5)
    plt.xlabel('Messages per seconds (msg/s)')

    #plt.yticks(ticks)
    plt.ylabel('Latency (seconds)')
    plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5))

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def rtt_box_plot(log, scale, outfile):
    fig, axes = plt.subplots()

    ys = [math.pow(10,e) for e in range(-7,0)]


    sns.set_style({'font.family':'serif', 'font.serif':'Times New Roman'})
    plt.rcParams["font.family"] = "serif"
    plt.rcParams["font.serif"] = 'Times New Roman'
    plt.rcParams['font.size'] = 13

    font = {'fontname':'Times New Roman', 'fontsize':13}


    medians = log.groupby(['framework','scenario','test','pipeline','payload','rate'])['value'].median().reset_index()

    g = sns.boxplot(data=log, x='rate', y='value', palette=palette, hue='framework',
        hue_order=hue_order,
        width=0.5,
        medianprops=dict(color="red", alpha=0.7),
        capprops=dict(linewidth=0.8),
        whiskerprops=dict(linewidth=0.8),
        flierprops=dict(marker="x", markersize=1),
        showfliers = False,
        boxprops=dict(linewidth=0.8))

    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5, **font)
    plt.xlabel('Messages rate (msg/s)', **font)

    plt.yticks(ys, **font)
    plt.ylabel('Latency (s)', **font)
    # plt.legend(title='Legend', loc='upper left', labels=labels)

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def rtt_violin_plot(log, scale, outfile):
    if scale != "log":
        raise ValueError("Volin Latency plot works only with log scale")


    #convert the data to log because:
    # Currently, violinplot does not compute the density
    # estimate in log space if the axis is log scaled;
    # it computes the density on a linear grid and then scales those values.
    # The same is true for boxplot, but boxplots are based on quantiles
    # and those do not change when log transformed (but note the
    # imbalance in outliers on the log plot version).
    # You'll need to log transform the data before giving it to either function.
    log['value']= log.apply(lambda line: np.log10(line.value), axis=1)

    fig, axes = plt.subplots()

    ys = [math.pow(10,e) for e in range(-7,0)]


    sns.set_style({'font.family':'serif', 'font.serif':'Times New Roman'})
    plt.rcParams["font.family"] = "serif"
    plt.rcParams["font.serif"] = 'Times New Roman'
    plt.rcParams['font.size'] = 13

    font = {'fontname':'Times New Roman', 'fontsize':13}


    medians = log.groupby(['framework','scenario','test','pipeline','payload','rate'])['value'].median().reset_index()

    ax = sns.violinplot(data=log, x='rate', y='value', palette=palette, hue='framework',
        inner="box",
        linewidth=0.8,
        bw=0.2,
        hue_order=hue_order,
        )
    #ax.set(ylim=(pow(10,-12), pow(10,-3)))

    sns.swarmplot(x='rate',
        y='value',
        data=medians,
        edgecolor='black',
        linewidth=1,
        size=4,
        dodge=True,
        ax=ax,
        hue='framework',
        hue_order=hue_order,
        palette=palette_medians,
        )

    # sns.boxplot(data=log, x='rate', y='value', palette=palette, hue='framework',
    #     width=0.1,
    #     medianprops=dict(color="red", alpha=0.7),
    #     capprops=dict(linewidth=0.8),
    #     whiskerprops=dict(linewidth=0.8),
    #     # flierprops=dict(marker="x", markersize=1),
    #     boxprops={'linewidth':0.8, 'zorder':2},
    #     showfliers=False,
    #     ax=axes,
    #     # dodge=False,
    #     )



    # if scale == 'log':
        # ax.set_yscale('log')


    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5, **font)
    plt.xlabel('Message rate (msg/s)', **font)

    plt.yticks(ys, **font)
    plt.ylabel('Latency (s)', **font)
    plt.legend(title='Legend', loc='upper left')

    handles, labels = ax.get_legend_handles_labels()
    ax.legend(handles[:2], labels[:2], title='Framework', loc='upper left')

    #ticker = mpl.ticker.EngFormatter(unit='')
    ticker = mpl.ticker.StrMethodFormatter("$10^{{{x:.0f}}}$")
    axes.yaxis.set_major_formatter(ticker)

    y_min, y_max = ax.get_ylim()
    print(f'min {y_min} max {y_max}')
    tick_range = np.arange(np.floor(y_min), y_max)
    ax.yaxis.set_ticks(tick_range)
    ax.yaxis.set_ticks([np.log10(x) for p in tick_range for x in np.linspace(10 ** p, 10 ** (p + 1), 10)], minor=True)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))

def throughput_stat_plot(log, scale, outfile):

    fig, axes = plt.subplots()

    ys = [math.pow(10,e) for e in range(0,7)]

    sns.set_style({'font.family':'serif', 'font.serif':'Times New Roman'})
    plt.rcParams["font.family"] = "serif"
    plt.rcParams["font.serif"] = 'Times New Roman'

    font = {'fontname':'Times New Roman'}

    g = sns.lineplot(data=log, x='label', y='value', palette=palette,
                ci=95, err_style='band', hue='framework',
                estimator=np.median)

    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.1)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.1, axis='y')

    plt.xticks(rotation=72.5, **font)
    plt.xlabel('Message size (bytes)', **font)
    plt.minorticks_off()
    plt.yticks(ys, **font)
    plt.ylabel('Throughput (msg/s)', **font)
    plt.legend(title='Legend', loc='center left', bbox_to_anchor=(1.0, 0.5)).remove()

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))

def throughput_box_plot(log, scale, outfile):
    fig, axes = plt.subplots()

    ys = [math.pow(10,e) for e in range(0,7)]

    sns.set_style({'font.family':'serif', 'font.serif':'Times New Roman'})
    plt.rcParams["font.family"] = "serif"
    plt.rcParams["font.serif"] = 'Times New Roman'

    font = {'fontname':'Times New Roman'}

    g = sns.boxplot(data=log, x='label', y='value', palette=palette, hue='framework',
        width=0.5,
        # medianprops=dict(color="red", alpha=0.7),
        # capprops=dict(linewidth=0.8),
        # whiskerprops=dict(linewidth=0.8),
        # flierprops=dict(marker="x", markersize=1),
        # boxprops=dict(linewidth=0.8),
        ax=axes,

        dodge=False)

    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.5)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0, axis='y')



    for line in axes.get_lines()[4::12]:
        line.set_color(palette['zenoh-flow'])
    for line in axes.get_lines()[10::12]:
        line.set_color(palette['erdos'])

    pl = sns.pointplot(data=log, x='label', y='value', hue='framework', ci=None,
            scale=0.3,  palette=palette, marker='O',  estimator=np.median)



    if scale == 'log':
        g.set_yscale('log')

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.5)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0, axis='y')

    pl.legend().remove()
    #pl.set(xlabel=None)
    #pl.set(ylabel=None)

    plt.xticks(rotation=72.5, **font)
    plt.xlabel('Message size (Byte)', **font)

    plt.yticks(ys, **font)
    plt.ylabel('Throughput (msg/s)', **font)
    plt.legend(title='Legend', loc='upper left').remove()

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))

def throughput_violin_plot(log, scale, outfile):
    fig, axes = plt.subplots()

    ys = [math.pow(10,e) for e in range(0,7)]

    sns.set_style({'font.family':'serif', 'font.serif':'Times New Roman'})
    plt.rcParams["font.family"] = "serif"
    plt.rcParams["font.serif"] = 'Times New Roman'
    plt.rcParams['font.size'] = 13

    font = {'fontname':'Times New Roman', 'fontsize':13}

    medians = log.groupby(['framework','scenario','test','pipeline','payload','label'])['value'].median().reset_index()

    ax = sns.violinplot(data=log, x='label', y='value', palette=palette, hue='framework',
        width=0.5,
        linewidth=0.8,
        ax=axes,
        dodge=False,
        hue_order=hue_order,
        )

    sns.swarmplot(x='label',
        y='value',
        data=medians,
        edgecolor='black',
        linewidth=0.2,
        size=2,
        dodge=False,
        ax=ax,
        hue='framework',
        hue_order=hue_order,
        palette=palette,
        )

    if scale == 'log':
        ax.set_yscale('log')


    # for line in axes.get_lines()[4::12]:
    #     line.set_color(palette['zenoh-flow'])
    # for line in axes.get_lines()[10::12]:
    #     line.set_color(palette['erdos'])

    pl = sns.pointplot(data=log,
        x='label',
        y='value',
        hue='framework',
        ci=None,
        scale=0.3,
        palette=palette,
        marker='O',
        estimator=np.median,
        ax=ax,
        hue_order=hue_order)

    pl.legend().remove()
    #pl.set(xlabel=None)
    #pl.set(ylabel=None)

    plt.grid(which='major', color='grey', linestyle='-', linewidth=0.5)
    plt.grid(which='minor', color='grey', linestyle=':', linewidth=0.01, axis='y')

    plt.xticks(rotation=72.5, **font)
    plt.xlabel('Message size (Byte)', **font)

    plt.yticks(ys, **font)
    plt.ylabel('Throughput (msg/s)', **font)
    plt.legend(title='Legend', loc='upper left').remove()

    ticker = mpl.ticker.EngFormatter(unit='')
    axes.yaxis.set_major_formatter(ticker)

    plt.tight_layout()
    fig.savefig(IMG_DIR.joinpath(outfile))


def rtt_raw_csv(log, outfile):

    d = {
        'framework':[],
        'rate':[],
        '20ile':[],
        'min':[],
        'mean':[],
        'max':[],
        '80ile':[],
        'cv':[],
        }

    df = pd.DataFrame(data=d)

    def mask(log, framework, rate):
        first_pass = log[log['framework']==framework]
        return first_pass[first_pass['rate']==rate]

    rates = np.unique(log['rate'])

    for r in rates:
        zf_data = mask(log, 'zenoh-flow', r)

        # Zenoh Flow calculations
        zf_min = np.amin(zf_data['value'])
        zf_max = np.amax(zf_data['value'])
        zf_mean = np.mean(zf_data['value'])
        zf_std = np.std(zf_data['value'])
        zf_20ile = np.percentile(zf_data['value'], 20)
        zf_80ile = np.percentile(zf_data['value'], 80)
        zf_cv = zf_std / zf_mean

        # Adding row
        row = {
                'framework': 'zenoh-flow',
                'rate': r,
                '20ile': zf_20ile,
                'min': zf_min,
                'mean': zf_mean,
                'max': zf_max,
                '80ile': zf_80ile,
                'cv': zf_cv
            }
        df = pd.concat([df, pd.DataFrame([row])], ignore_index=True)

    for r in rates:
        erdos_data = mask(log, 'erdos', r)

        # ERDOS calculations
        e_min = np.amin(erdos_data['value'])
        e_max = np.amax(erdos_data['value'])
        e_mean = np.mean(erdos_data['value'])
        e_std = np.std(erdos_data['value'])
        e_20ile = np.percentile(erdos_data['value'], 20)
        e_80ile = np.percentile(erdos_data['value'], 80)
        e_cv = e_std / e_mean

        # Adding row
        row = {
                'framework': 'erdos',
                'rate': r,
                '20ile': e_20ile,
                'min': e_min,
                'mean': e_mean,
                'max': e_max,
                '80ile': e_80ile,
                'cv': e_cv
            }
        df = pd.concat([df, pd.DataFrame([row])], ignore_index=True)

    df.to_csv(outfile)


def main():
    parser = argparse.ArgumentParser(description='Parse zenoh flow performance results')
    parser.add_argument('-k','--kind', help='Kind of the tests', required=False, choices=['latency', 'throughput'], default='latency')
    parser.add_argument('-d','--data', help='Logs directory', required=True, type=str)
    parser.add_argument('-p','--scenario', help='single or multi', choices=['single', 'multi'], required=False)
    parser.add_argument('-t','--type', help='Plot type', choices=['stat', 'time', 'ecdf', 'pdf', 'box','violin','raw'], default='stat', required=False)
    parser.add_argument('-s','--scale', help='Plot scale', choices=['log', 'lin'], default='log', required=False)
    parser.add_argument('-r','--rate', help='Filter for this rate', required=False, type=float)
    parser.add_argument('-o','--output', help='Output file name', required=False, type=str, default='plot.pdf')

    args = vars(parser.parse_args())
    data = args['data']
    print(f'[ START ] Processing data in { data }')

    if not os.path.exists(IMG_DIR):
        os.makedirs(IMG_DIR)

    log = prepare(args['data'], args['kind'])
    print(f'[ STEP1 ] Read a total of {log.size} samples')
    log = filter(log, args.get('scenario', None), args.get('rate', None))
    print(f'[ STEP2 ] After filtering we have {log.size} samples')
    if log.size == 0:
        print(f'[ ERR ] Cannot continue without samples!')
        exit(-1)


    if args['kind'] == 'latency':
        if args['type'] == 'stat':
            rtt_stat_plot(log, args['scale'], args['output'])
        elif args['type'] == 'box':
            rtt_box_plot(log, args['scale'], args['output'])
        elif args['type'] == 'violin':
            rtt_violin_plot(log, args['scale'], args['output'])
        elif args['type'] == 'time':
            rtt_time_plot(log, args['scale'], args['output'])
        elif args['type'] == 'ecdf':
            rtt_ecfd_plot(log, args['scale'], args['output'])
        elif args['type'] == 'pdf':
            rtt_pdf_plot(log, args['scale'], args['output'])
        elif args['type'] == 'raw':
            rtt_raw_csv(log, args['output'])

    elif args['kind'] == 'throughput':
        if args['type'] == 'stat':
            throughput_stat_plot(log, args['scale'], args['output'])
        if args['type'] == 'box':
            throughput_box_plot(log, args['scale'], args['output'])
        elif args['type'] == 'time':
            print('Not implemented.')
            # rtt_time_plot(log, args['scale'], args['output'])
        elif args['type'] == 'ecdf':
            print('Not implemented.')
            # rtt_ecfd_plot(log, args['scale'], args['output'])
        elif args['type'] == 'pdf':
            print('Not implemented.')
        elif args['type'] == 'violin':
            throughput_violin_plot(log, args['scale'], args['output'])
            # rtt_pdf_plot(log, args['scale'], args['output'])

    out = IMG_DIR.joinpath(args['output'])
    print(f'[  DONE ] File saved to { out }')




if __name__=='__main__':
    main()