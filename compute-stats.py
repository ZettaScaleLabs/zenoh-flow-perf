#!/usr/bin/env python3
import os
from io import StringIO
import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
import math

import pandas as pd

# import modin.pandas as pd
from pathlib import Path
import seaborn as sns
import sys
import argparse
import json

# import ray

# ray.init()

palette = {
    "zenoh-flow": "tab:blue",
    "erdos": "tab:green",
}

styles = {
    "single": (0, 0),
    "multi": (1, 1),
}


# palette = 'bright' #sns.color_palette("bright", 6) #'plasma'
IMG_DIR = Path("img")


def pairwise(data):
    l = iter(data)
    return zip(l, l)


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
    if line.unit == "s":
        return line.value
    if line.unit == "ms":
        return line.value / pow(10,3)
    if line.unit == "us":
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
                log = pd.concat([log, pd.read_csv(l)])
    return log


def mask_first_and_last(x):
    mask = [True] * len(x)
    mask[0] = False
    mask[1] = False
    mask[-2] = False
    mask[-1] = False
    return mask


def filter(log, process, msgs=None, pipeline=None):
    layers = log["framework"].unique()

    # filtering if multi process or same process\
    log = log[log["scenario"].isin([process])]

    # filtering is msg/s is set
    if msgs is not None:
        log = log[log["rate"] == msgs]

    if pipeline is not None:
        log = log[log["pipeline"] == pipeline]

    return log


def prepare(log_dir, kind):
    log = read_log(log_dir)

    # filtering by kind of test
    log = log[log["test"] == kind]

    log["value"] = pd.to_numeric(log["value"], errors="coerce")
    # if kind == "latency":
    #     # Remove first and last two samples of every test
    #     mask = log.groupby(
    #         ["framework", "scenario", "pipeline", "test", "payload", "rate"]
    #     ).transform(mask_first_and_last)["value"]
    # elif kind == "throughput":
    #     # Remove first and last two samples of every test
    #     mask = log.groupby(
    #         ["framework", "scenario", "pipeline", "test", "payload", "rate"]
    #     ).transform(mask_first_and_last)["value"]
    # log = log.loc[mask]

    if kind == "latency":
        # this converts everything to seconds, data is expected as micro seconds
        log["value"] = log.apply(convert_value, axis=1)
        log["label"] = [interval_label(v) for k, v in log["rate"].iteritems()]
        log.sort_values(by="rate", inplace=True, ascending=False)

    elif kind == "throughput":
        log["label"] = [bytes_label(v) for k, v in log["payload"].iteritems()]
        log.sort_values(by="payload", inplace=True)

    log["framework"] = log["framework"].astype(str)

    log = log.reset_index()

    return log


def print_statistics(log):
    pass


def main():
    parser = argparse.ArgumentParser(description="Parse zenoh flow performance results")
    parser.add_argument(
        "-k",
        "--kind",
        help="Kind of the tests",
        required=False,
        choices=["latency", "throughput"],
        default="latency",
    )
    parser.add_argument("-d", "--data", help="Logs directory", required=True, type=str)
    parser.add_argument(
        "-p",
        "--scenario",
        help="single or multi",
        choices=["single", "multi"],
        required=False,
    )
    parser.add_argument(
        "-r", "--rate", help="Filter for this rate", required=False, type=float
    )

    args = vars(parser.parse_args())
    data = args["data"]
    print(f"[ START ] Processing data in { data }")

    log = prepare(args["data"], args["kind"])
    print(f"[ STEP1 ] Read a total of {log.size} samples")
    log = filter(log, args.get("scenario", None), args.get("rate", None))
    print(f"[ STEP2 ] After filtering we have {log.size} samples")
    if log.size == 0:
        print(f"[ ERR ] Cannot continue without samples!")
        exit(-1)

    frameworks = log["framework"].unique()
    stats = {}

    for f in frameworks:
        local_log = log[log["framework"] == f]

        rates = local_log["rate"].unique()
        d = {}
        for r in rates:
            rate_log = local_log[local_log["rate"] == r]

            d[r] = {
                "min": np.min(rate_log["value"]),
                "mean": np.mean(rate_log["value"]),
                "median": np.median(rate_log["value"]),
                "99th": np.percentile(rate_log["value"], 99),
                "max": np.max(rate_log["value"]),
                "stddev": np.std(rate_log["value"]),
                "cv": np.std(rate_log["value"])/np.mean(rate_log["value"]),

            }
        stats[f] = d

    for k, v in stats.items():
        print(f"Framework: {k}")
        for r, data in v.items():
            print(
                f"Rate: {r}, Min: {data['min']}, Mean: {data['mean']}, Median: {data['median']}, 99th-tile: {data['99th']}, Max: {data['max']}, Std-Dev: {data['stddev']}, Cv: {data['cv']}"
            )


if __name__ == "__main__":
    main()