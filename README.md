# Async Channel Benchmarks

![Oneshot Send](plots/oneshot-send.svg)
![Oneshot Recv](plots/oneshot-recv.svg)
![MPMC Send](plots/mpmc-send.svg)
![MPMC Recv](plots/mpmc-recv.svg)

Run `cargo run -- --bench` to run the benchmarks. Run `cd process` and `cargo run` to process the
data and put it into the `data` directory. Run `plot.R` in the `plots` folder to generate the plot
SVGs.
