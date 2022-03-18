#!/usr/bin/env python3
"""Simple benchmarking framework for bar charts"""

import functools
from pathlib import Path
import statistics
import timeit
from typing import Any, Callable, cast, Dict, List, Optional, Tuple, Union

import matplotlib.pyplot as plt
import memory_profiler

class Kernel:
    Callback = Union[Callable[[Any], Any], Callable[[], Any]]
    """A kernel to be called later"""
    def __init__(self, 
        setup: Callable[[], Any],
        func: Callback,
        name: str):

        self.setup = setup
        self.func = func
        self.name = name

class Benchmark:
    """A collection of runs (kernel) that make up a benchmark"""
    def __init__(self):
        self.kernels: List[Kernel] = []

collections: Dict[str, Benchmark] = {}

def benchmark(_func=None, *, name: Optional[str]= None, collection:Optional[str]=None, with_classes:List[Any]=[]):
    """
    Mark a function to be run for benchmarking.

    Note: Generally this makes just one execution to be grouped in a collection,
        however, declaring "with_classes" makes it like a collection of
        executions (a benchmark in itself) each one called with a different class.

    keyword arguments:
    name -- The name of this run of benchmark. Ignored if with_classes is not empty.
        Otherwise, if left empty the name of the function with "_"
        transformed to " " is taken.
        
    collection -- Benchmarks are stored in collections. The default is 
        "default", unless with_classes is not empty, in which case is the name
        of the function.

    with_classes -- A list of classes to be used as the argument for the benchmark function.
    """

    def decorator_benchmark(func):
        """Internal wrapper that helps with the difference in having arguments
        or no arguments in the decorator."""
        if collection is None:
            if len(with_classes) > 0:
                col_name = func.__name__.replace("_", " ").capitalize()
            else:
                col_name = "default"
        else: col_name = collection
        
        if col_name not in collections:
            collections[col_name] = Benchmark()

        if len(with_classes)== 0:
            ker_name = name or  func.__name__.replace("_", " ").capitalize()
            
            def do_nothing(*args): pass

            collections[col_name].kernels.append(Kernel(do_nothing, func, ker_name))
        else:
            for cls in with_classes:
                def make_setup(cls):
                    """We need so much indirection to force the lambda to store cls itself
                    (otherwise keeps the cls which will be changing and end ups being the 
                    last class in with_classes)"""
                    cls_to_test=cls

                    def inner_setup(): return cls_to_test()
                    return inner_setup

                collections[col_name].kernels.append(Kernel(make_setup(cls), func, cls.__name__))


        

        return func

    if _func is None:
        return decorator_benchmark
    else:
        return decorator_benchmark(_func)


def run():
    """Run all the benchmarks and save the results. Call this at the end after
    every benchmark has been marked and all the configuration is set"""

    svg_out = Path("figures")
    ticks = 6
    bar_color = (0.121568627, 0.466666667, 0.705882353)
    bar_color_err= (0.066666667, 0.266666667, 0.403921569)
    repeat = 5

    out_md = "# Benchmark\n\n"

    svg_out.mkdir(exist_ok=True)

    def time_kernel(kernel: Kernel) -> float:
        init_data = kernel.setup()
        if init_data is not None:
            fun: Callable[[], Any] = functools.partial(kernel.func,init_data)
        else:
            fun = cast(Callable[[], Any], kernel.func)

        return timeit.timeit(fun, number=1)

    def mem_of_kernel(kernel: Kernel) -> float:
        init_data = kernel.setup()
        if init_data is not None:
            tuple_call: Tuple[Kernel.Callback, List[Callable[[], Any]], Dict] = (kernel.func, [kernel.setup()], {})            
        else:
            tuple_call = (kernel.func, [], {})

        return max(memory_profiler.memory_usage(tuple_call))
        

    for name, benchmark in collections.items():
        print(f"Running benchmark {name}")
        def run_kernel(kernel: Kernel):
            print(f"Running kernel {kernel.name}")
            return ([time_kernel(kernel) for _ in range(repeat)], kernel.name, mem_of_kernel(kernel))

        times = map(run_kernel, benchmark.kernels)
        chart_vals = list(map(lambda args: (min(args[0]), statistics.stdev(args[0]), args[1], args[2]), times))
        chart_lists = list(map(list, zip(*chart_vals)))
        max_val = max(map(lambda args: args[0] + args[1], chart_vals))
    
        pos = list(range(0,len(chart_lists[0])))

        # Setup the plot to be pretty
        ax = plt.subplot(111)
        ax.spines["top"].set_visible(False)    
        ax.spines["bottom"].set_visible(False)    
        ax.spines["right"].set_visible(False)    
        ax.spines["left"].set_visible(False)
        ax.get_xaxis().tick_bottom()    
        ax.get_yaxis().tick_left()
        
        step = max_val / (ticks -1)

        # Here "{:.2e}" means that we format with 2 decimals and use scientific notation
        plt.yticks([x * step for x in range(0, ticks)], ["{:.2e}".format(x * step) for x in range(0, ticks)], fontsize=12)
        plt.xticks(pos, chart_lists[2], fontsize=14)  
        
        # Limit the range of the plot to only where the data is.    
        plt.ylim(0, max_val)
        plt.margins(x=-0.35 *  (2/ (len(chart_lists[0])+2)), y=0.00)

        # Provide tick lines across the plot to help your viewers trace along    
        # the axis ticks. Make sure that the lines are light and small so they    
        # don't obscure the primary data lines.    
        for y in range(0,ticks):
            r = range(-1, len(chart_lists[0])+1)
            plt.plot(r, [y * step for _ in r] , "--", lw=0.5, color="black", alpha=0.3)  

        # Plot the bars
        plt.bar(pos, chart_lists[0], yerr=chart_lists[1], align='center', alpha=1, ecolor=bar_color_err, capsize=10, color=bar_color, capstyle='round')

        svg_path = svg_out/f"{name}.svg"
        plt.savefig(svg_path, bbox_inches="tight")

        out_md += f"## {name.capitalize()}\n\n![{name}]({svg_path})\n\n"

        # TODO: Write memory usage as chart
        names_and_times = zip(chart_lists[2], chart_lists[3])
        mem_of_kernels = functools.reduce(
            lambda md, n_m: f"{md}{n_m[0]}: {n_m[1]:10.2f}Mb\n",
            names_and_times,
            ""
        )
        out_md += f"### Memory usage\n\n{mem_of_kernels}\n"

        plt.clf()

    Path("benchmark.md").write_text(out_md)

def conf(collection: str):
    """
    Configure a collection
    """

    pass

if __name__ == "__main__":
    import time

    class OtherClass: pass
    class TestClass2: pass
    
    @benchmark(collection="other")
    def test_hello():
        print("Hello world")
    
    @benchmark
    def test_hello2():
        print("Hello default")

    @benchmark
    def test_sleep():
        time.sleep(0.0000000000000000000000000000001)

    @benchmark(with_classes=[OtherClass, TestClass2])
    def test_classes(cls):
        print(cls.__class__.__name__)

    # Mandatory: Run will run all benchmarks and output the information
    run()