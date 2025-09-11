# varnishotel

**Disclaimer** This project is work in progress and not suitable for production usage. 

## What is varnishotel?

varnishotel is a combined observability solution for the [Varnish Cache](https://varnish-cache.org/)
that exports telemetry data (logs, traces and metrics) to [OpenTelemetry](https://opentelemetry.io/) compatible backends. 

This project was inspired by [this](https://info.varnish-software.com/blog/tech-preview-varnish-otel) blog 
entry by Varnish software and takes it open source.

## Overview

The main component of varnishotel is a sidecar running on the same host as a Varnish instance. It uses
tools that come with an installation of Varnish such as `varnishstat` and `varnishlog` to gather data and
converts them into OpenTelemetry compatible outputs. 

## Design

The varnishotel binary runs on the same host as the Varnish instance it collects its data from. There are two
main _collectors_ for gathering data from Varnish:

- The metrics collector runs `varnishstat -j -1` to scrape metrics from Varnish in JSON format, parse it and 
  convert the result to Prometheus compatible metrics. 
- The log/trace collector runs `varnishlog` or `varnishncsa` to scrape logs from Varnish, parse it and convert
  the result to traces and correlate subrequests.
