# Trace naming conventions

## Background

Each request to the Varnish instance has multiple (sub)requests:

- The client request 
- Subrequests with `client` side 
- Subrequests with `backend` side

This document describes how traces are named in `varnishotel`.

## Design

**TL;DR** A typical trace looks like this:

- `Varnish request processing`
- `Varnish client miss`
- `Varnish to mybackend fetch`

There are some special cases:

- Cache hits won't produce a backend trace
- Using ESI will produce multiple blocks of `Varnish client miss` and `Varnish to mybackend fetch` traces

### Client request

The client request describes what Varnish initially receives from the client. Client requests
are always named `Varnish request processing`. 

### Subrequest: client side

Client-side subrequests describe the start of a backend request (e.g. ESI request) from the VCL 
`vcl_recv` routine. They are named `Varnish <side> <handling>` where 

- `side` is one of `backend` or `client`
- `handling` is one of `"hit" | "miss" | "pass" |"pipe" |
              "streaming-hit" | "fail" | "synth"
              "abandon" | "fetched" | "error"
              "retry" | "restart"` 

**Rationale:** Tells the user which direction the request was made to and if it was a cache hit, 
pass or error. 

### Subrequest: backend side

Backend request describe the request to a backend. Therefor, they are named `Varnish to <backend> fetch`.

**Rationale:** The operation name indicates which backend was fetched. 

#### Backend naming

The processor will ensure that the name is cleaned, e.g. when using VMODs such as `vmod-dynamic` that create dynamic backends.
Backends in `vmod-dynamic` for example are called `<vcl_name>(<address>:<port>)` and are converted to `<vcl_name>`. 
Address information is still available in the attributes per span. 