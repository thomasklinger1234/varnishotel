vcl 4.1;

import std;
import directors;
import dynamic;
import header;
import cookie;
import var;

backend default none;

# create a director that can find backends on-the-fly
sub vcl_init {
    new dyn_default = dynamic.director(
        ttl = 30s,
        domain_usage_timeout = 3s,
        first_lookup_timeout = 3s,
        max_connections = 10000,
        between_bytes_timeout = 1s,
        first_byte_timeout = 1s,
        connect_timeout = 1s,
        retry_after = 10s,
        keep = 100,
    );
    new dyn_default_dir = directors.round_robin();
    dyn_default_dir.add_backend(dyn_default.backend(
        std.getenv("VARNISH_BACKEND_HOST"),
        std.getenv("VARNISH_BACKEND_PORT"),
    ));
}

sub vcl_recv {
    set req.http.host = std.getenv("VARNISH_BACKEND_HOST");
    set req.backend_hint = dyn_default_dir.backend();
}

sub vcl_backend_response {
    set beresp.do_esi = true;
    set beresp.do_gzip = true;
}