# `adb_loader`: Produce constant load on an ArangoDB cluster

This project provides a program, which can produce a constant load on
an ArangoDB cluster.

The whole program runs in a single process and is configured via a single
YAML configuration file (see `config.yaml`). Other than that, it runs
fully automatically without any further user interaction.

It writes a log to standard output and provides metrics on some designated
port in Prometheus format.

It also collects all abnormal events in a special alert log.

The configuration file has comments to document itself.

This program is written in Rust with the help of LLMs.

THIS IS STILL WORK IN PROGRESS.
