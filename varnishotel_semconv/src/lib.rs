/// Uniquely identifies the Varnish transaction id.
pub const VARNISH_VXID: &str = "varnish.vxid";

/// The Varnish VCL name.
pub const VARNISH_VCL: &str = "varnish.vcl";

/// The Varnish request side (`client` or `backend`).
pub const VARNISH_SIDE: &str = "varnish.side";

/// The Varnish backend name.
pub const VARNISH_BACKEND_NAME: &str = "varnish.backend.name";

/// Indicates if the backend connection was reused by Varnish from
/// an existing pool of connections.
pub const VARNISH_BACKEND_CONN_REUSED: &str = "varnish.backend.conn_reused";
