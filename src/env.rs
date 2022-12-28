use std::str::FromStr;

type ENV = &'static str;

pub const TELOXIDE_TOKEN: ENV = "TELOXIDE_TOKEN";

pub fn parse_var<T: FromStr>(var: &'static str) -> Option<T> {
  dotenvy::var(var).ok().and_then(|x| x.parse().ok())
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

  failed.then(|| {
    error!("Not all .env args are set");
    panic!("Not all .env args are set");
  });
}
