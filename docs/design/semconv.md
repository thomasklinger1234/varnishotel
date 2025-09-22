# Semantic conventions for Varnish

## Introcuction

Each span needs to be annotated with metadata to be useful. OpenTelemetry has [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/general/trace/)
for every signal. This document describes attributes that are specific to Varnish and its request and 
response handling.

## Design

The following attributes are added 

| Attribute                     | Type    | Description                                   | Examples           |
|-------------------------------|---------|-----------------------------------------------|--------------------|
| `varnish.vxid`                | String  | ID of the transaction.                        | `213`              |
| `varnish.side`                | String  | Request handling side.                        | `client`;`backend` |
| `varnish.vcl`                 | String  | Name of the VCL that handles the request.     | `boot`             |
| `varnish.backend.name`        | String  | Name of the backend that handles the request. | `default`          |
| `varnish.backend.conn_reused` | Boolean | Indicates if the connection has been reused.  | `false`            |

The Rust package `varnishotel_semconv` is added to hold those attributes. 