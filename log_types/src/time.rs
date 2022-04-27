use std::ops::RangeInclusive;

/// A date-time represented as nanoseconds since unix epoch
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Time(i64);

impl Time {
    // #[inline]
    // pub fn now() -> Self {
    //     Self(nanos_since_epoch())
    // }

    #[inline]
    pub fn nanos_since_epoch(&self) -> i64 {
        self.0
    }

    #[inline]
    pub fn from_ns_since_epoch(ns_since_epoch: i64) -> Self {
        Self(ns_since_epoch)
    }

    #[inline]
    pub fn from_us_since_epoch(us_since_epoch: i64) -> Self {
        Self(us_since_epoch * 1_000)
    }

    #[inline]
    pub fn from_seconds_since_epoch(secs: f64) -> Self {
        Self::from_ns_since_epoch((secs * 1e9).round() as _)
    }

    /// Human-readable formatting
    pub fn format(&self) -> String {
        let nanos_since_epoch = self.nanos_since_epoch();
        let years_since_epoch = nanos_since_epoch / 1_000_000_000 / 60 / 60 / 24 / 365;

        if 50 <= years_since_epoch && years_since_epoch <= 150 {
            use chrono::TimeZone as _;
            let datetime = chrono::Utc.timestamp(
                nanos_since_epoch / 1_000_000_000,
                (nanos_since_epoch % 1_000_000_000) as _,
            );

            if datetime.date() == chrono::offset::Utc::today() {
                datetime.format("%H:%M:%S%.6fZ").to_string()
            } else {
                datetime.format("%Y-%m-%d %H:%M:%S%.6fZ").to_string()
            }
        } else {
            let secs = nanos_since_epoch as f64 * 1e-9;
            // assume relative time
            format!("{:+.03}s", secs)
        }
    }

    #[inline]
    pub fn lerp(range: RangeInclusive<Time>, t: f32) -> Time {
        let (min, max) = (range.start().0, range.end().0);
        Self(min + ((max - min) as f64 * (t as f64)).round() as i64)
    }
}

impl std::fmt::Debug for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format().fmt(f)
    }
}

impl std::ops::Sub for Time {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Time) -> Duration {
        Duration(self.0.saturating_sub(rhs.0))
    }
}

impl std::ops::Add<Duration> for Time {
    type Output = Time;

    #[inline]
    fn add(self, duration: Duration) -> Self::Output {
        Time(self.0.saturating_add(duration.0))
    }
}

impl std::ops::AddAssign<Duration> for Time {
    #[inline]
    fn add_assign(&mut self, duration: Duration) {
        self.0 = self.0.saturating_add(duration.0);
    }
}

impl TryFrom<std::time::SystemTime> for Time {
    type Error = std::time::SystemTimeError;

    fn try_from(time: std::time::SystemTime) -> Result<Time, Self::Error> {
        time.duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|duration_since_epoch| Time(duration_since_epoch.as_nanos() as _))
    }
}

// ----------------------------------------------------------------------------

/// A signed duration represented as nanoseconds since unix epoch
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Duration(i64);

impl Duration {
    pub const MAX: Duration = Duration(std::i64::MAX);

    #[inline]
    pub fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    #[inline]
    pub fn from_secs(secs: f32) -> Self {
        Self::from_nanos((secs * 1e9).round() as _)
    }

    #[inline]
    pub fn as_nanos(&self) -> i64 {
        self.0
    }

    #[inline]
    pub fn as_secs_f32(&self) -> f32 {
        self.0 as f32 * 1e-9
    }

    #[inline]
    pub fn as_secs_f64(&self) -> f64 {
        self.0 as f64 * 1e-9
    }

    pub fn exact_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const NANOS_PER_SEC: i64 = 1_000_000_000;
        const SEC_PER_MINUTE: i64 = 60;
        const SEC_PER_HOUR: i64 = 60 * SEC_PER_MINUTE;
        const SEC_PER_DAY: i64 = 24 * SEC_PER_HOUR;

        let total_nanos = if self.0 < 0 {
            // negative duration
            write!(f, "-")?;
            std::ops::Neg::neg(*self).0 // handle negation without overflow
        } else {
            self.0
        };

        let whole_seconds = total_nanos / NANOS_PER_SEC;
        let nanos = total_nanos - NANOS_PER_SEC * whole_seconds;

        let mut seconds_remaining = whole_seconds;
        let mut did_write = false;

        let days = seconds_remaining / SEC_PER_DAY;
        if days > 0 {
            write!(f, "{}d", days)?;
            seconds_remaining -= days * SEC_PER_DAY;
            did_write = true;
        }

        let hours = seconds_remaining / SEC_PER_HOUR;
        if hours > 0 {
            if did_write {
                write!(f, " ")?;
            }
            write!(f, "{}h", hours)?;
            seconds_remaining -= hours * SEC_PER_HOUR;
            did_write = true;
        }

        let minutes = seconds_remaining / SEC_PER_MINUTE;
        if minutes > 0 {
            if did_write {
                write!(f, " ")?;
            }
            write!(f, "{}m", minutes)?;
            seconds_remaining -= minutes * SEC_PER_MINUTE;
            did_write = true;
        }

        const MAX_MILLISECOND_ACCURACY: bool = true;
        const MAX_MICROSECOND_ACCURACY: bool = true;

        if seconds_remaining > 0 || nanos > 0 || !did_write {
            if did_write {
                write!(f, " ")?;
            }

            if nanos == 0 {
                write!(f, "{}s", seconds_remaining)?;
            } else if MAX_MILLISECOND_ACCURACY || nanos % 1_000_000 == 0 {
                write!(f, "{}.{:03}s", seconds_remaining, nanos / 1_000_000)?;
            } else if MAX_MICROSECOND_ACCURACY || nanos % 1_000 == 0 {
                write!(f, "{}.{:06}s", seconds_remaining, nanos / 1_000)?;
            } else {
                write!(f, "{}.{:09}s", seconds_remaining, nanos)?;
            }
        }

        Ok(())
    }
}

impl std::ops::Neg for Duration {
    type Output = Duration;

    #[inline]
    fn neg(self) -> Duration {
        // Handle negation without overflow:
        if self.0 == std::i64::MIN {
            Duration(std::i64::MAX)
        } else {
            Duration(-self.0)
        }
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.exact_format(f)
    }
}