/// Upper bound on the `name` argument accepted by the greeting port.
/// Keeps responses bounded in size and exercises the `BadRequest` path.
pub const MAX_GREET_NAME_LEN: usize = 64;
