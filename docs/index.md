# varnishotel: Export OpenTelemetry data from Varnish

varnishotel operates as a sidecar along a [Varnish](https://varnish-cache.org) instance
and exports [OpenTelemetry](https://opentelemetry.io/) telemetry data.

Its notable features encompass:

- **Traces:** Provides insights in request processing.
- **Logs:** Not implemented.
- **Metrics:** Not implemented.

## Why OpenTelemetry?

OpenTelemetry emerges as the standard for exporting operational telemetry data
and is a CNCF top-level project. 

## Why OpenTelemetry in Varnish?

Inspired by the blog post [Tech Preview: varnish-otel](https://info.varnish-software.com/blog/tech-preview-varnish-otel)
by Varnish Software, this project aims to combine the power of `varnishstat` and `varnishlog` (or similar tools)
with the standard of OpenTelemetry to serve as a single place of observability in Varnish deployments.

## Challenges for Observability in Varnish

Varnish provides powerful monitoring tools like `varnishstat` and `varnishlog` out of the
box. However, they are not capable of exposing the data to external monitoring systems and 
require wrappers around them such as:

- [jonnenauha/prometheus_varnish_exporter](https://github.com/jonnenauha/prometheus_varnish_exporter) for exposing
  `varnishstat` metrics to Prometheus
- [varnish/varnishlog-json](https://github.com/varnish/varnishlog-json) to convert Varnish internal logs
  to JSON. This approach requires an external collector like fluent-bit, Logstash or vector to execute the program
  and export the logs for long-term storage. 



