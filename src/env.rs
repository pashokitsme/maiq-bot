use std::str::FromStr;

macro_rules! env_var {
  ($var_name: ident, $env_name: literal) => {
    pub const $var_name: &'static str = $env_name;
  };
  ($var_name: ident) => {
    pub const $var_name: &'static str = stringify!($var_name);
  };
}

env_var!(TELOXIDE_TOKEN);
env_var!(DEV_ID);
env_var!(API_HOST);
env_var!(DB_URL, "DATABASE_CONNECTION_URL");
env_var!(DEFAULT_DB, "DEFAULT_DATABASE_NAME");

pub fn parse_var<T: FromStr>(var: &'static str) -> Option<T> {
  self::var(var).and_then(|x| x.parse().ok())
}

pub fn var(var: &'static str) -> Option<String> {
  dotenvy::var(var).ok()
}

pub fn check<T: FromStr>(var: &'static str) -> bool {
  parse_var::<T>(var)
    .is_none()
    .then(|| error!("Var {}: {} is not present", var, std::any::type_name::<T>().split("::").last().unwrap()))
    .is_none()
}

pub fn check_env_vars() {
  info!("Validating .env vars");
  let mut failed = false;

  failed |= !check::<String>(TELOXIDE_TOKEN);
  failed |= !check::<String>(API_HOST);
  failed |= !check::<String>(DB_URL);
  failed |= !check::<String>(DEFAULT_DB);

  failed.then(|| {
    error!("Not all .env args are set");
    panic!("Not all .env args are set");
  });
}
