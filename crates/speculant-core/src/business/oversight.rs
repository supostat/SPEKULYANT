use crate::zone::TrusteeConduct;

/// Кто присматривает за делом в месяце расчёта: хозяин или, пока он сидит, доверенное лицо (§12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oversight {
    Owner,
    Trustee(TrusteeConduct),
}
