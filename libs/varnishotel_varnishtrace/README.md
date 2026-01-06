# `varnishotel_varnishtrace`

This crate implements gathering of OTEL traces. It includes logic that:

- Runs `varnishlogjson`
- Parses the output and converts it into an OTEL trace