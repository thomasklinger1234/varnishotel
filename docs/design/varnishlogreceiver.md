# varnishlogreceiver

## Introduction

This document describes the design of the `varnishlogreceiver` component in varnishotel. This 
receiver is responsible for collecting request logs from Varnish and converting those logs into 
OpenTelemetry traces. 

## Design

`varnishlogreceiver` is a component that wraps the [varnishlog-json](https://github.com/varnish/varnishlog-json)
binary and converts it into [traces](https://opentelemetry.io/docs/concepts/signals/traces/). Traces
are then enriched with attributes ,both standard [semantic conventions](https://opentelemetry.io/docs/specs/semconv/general/trace/) and custom
ones.

The `varnishlog-json` binary is chosen over `varnishlog` or `varnishncsa` because it groups requests and outputs
them in an appropriate format, i.e. a single line of JSON containing both backend and request information. By using the `-g request`
flag, each request is a JSON array that contains the client request as the first item and other requests as the remainder:

```
$ varnishlog-json -g request -p 
[
    {"side": "client", ... }, # client-facing request
    {"side": "backend", ...}, 
    ...
] 
```

This format makes it ideal for parsing and correlating a single frontend request with the corresponding
backend requests without having to parse `varnishlog` output. 

The main loop will stream the stdout of the `varnishlog-json` process and process it line by line. Each line
is then processed according to the following steps:

- Extract the first item of the request and set it as the root span 
- For the remainder of items:
    - Create a sub span
    - Enrich the subspan with attributes
- Close the root span