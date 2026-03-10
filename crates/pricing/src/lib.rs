#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct HigherOrderGreeks {
    pub vanna: f64,
    pub vomma: f64,
    pub charm: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SurfacePoint {
    pub strike: f64,
    pub maturity_years: f64,
    pub iv: f64,
}

pub fn black_scholes_price(
    spot: f64,
    strike: f64,
    maturity_years: f64,
    rate: f64,
    volatility: f64,
    kind: OptionKind,
) -> f64 {
    let (d1, d2) = d1d2(spot, strike, maturity_years, rate, volatility);
    let discount = (-rate * maturity_years).exp();
    match kind {
        OptionKind::Call => spot * norm_cdf(d1) - strike * discount * norm_cdf(d2),
        OptionKind::Put => strike * discount * norm_cdf(-d2) - spot * norm_cdf(-d1),
    }
}

pub fn black_scholes_greeks(
    spot: f64,
    strike: f64,
    maturity_years: f64,
    rate: f64,
    volatility: f64,
    kind: OptionKind,
) -> Greeks {
    let (d1, d2) = d1d2(spot, strike, maturity_years, rate, volatility);
    let sqrt_t = maturity_years.sqrt();
    let pdf = norm_pdf(d1);
    let discount = (-rate * maturity_years).exp();

    let delta = match kind {
        OptionKind::Call => norm_cdf(d1),
        OptionKind::Put => norm_cdf(d1) - 1.0,
    };
    let gamma = pdf / (spot * volatility * sqrt_t);
    let vega = spot * pdf * sqrt_t;
    let theta = match kind {
        OptionKind::Call => {
            -(spot * pdf * volatility) / (2.0 * sqrt_t) - rate * strike * discount * norm_cdf(d2)
        }
        OptionKind::Put => {
            -(spot * pdf * volatility) / (2.0 * sqrt_t) + rate * strike * discount * norm_cdf(-d2)
        }
    };
    let rho = match kind {
        OptionKind::Call => strike * maturity_years * discount * norm_cdf(d2),
        OptionKind::Put => -strike * maturity_years * discount * norm_cdf(-d2),
    };

    Greeks {
        delta,
        gamma,
        theta,
        vega,
        rho,
    }
}

pub fn higher_order_greeks(
    spot: f64,
    strike: f64,
    maturity_years: f64,
    rate: f64,
    volatility: f64,
) -> HigherOrderGreeks {
    let (d1, d2) = d1d2(spot, strike, maturity_years, rate, volatility);
    let sqrt_t = maturity_years.sqrt();
    let pdf = norm_pdf(d1);
    let vega = spot * pdf * sqrt_t;
    let vanna = (vega / spot) * (1.0 - d1 / (volatility * sqrt_t));
    let vomma = vega * d1 * d2 / volatility;
    let charm = -pdf
        * ((2.0 * rate * maturity_years - d2 * volatility * sqrt_t)
            / (2.0 * maturity_years * volatility * sqrt_t));

    HigherOrderGreeks {
        vanna,
        vomma,
        charm,
    }
}

pub fn implied_volatility(
    target_price: f64,
    spot: f64,
    strike: f64,
    maturity_years: f64,
    rate: f64,
    kind: OptionKind,
) -> Option<f64> {
    let mut low = 1e-4;
    let mut high = 5.0;

    for _ in 0..100 {
        let mid = 0.5 * (low + high);
        let model = black_scholes_price(spot, strike, maturity_years, rate, mid, kind);
        let error = model - target_price;

        if error.abs() < 1e-8 {
            return Some(mid);
        }
        if error > 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }

    Some(0.5 * (low + high))
}

pub fn detect_surface_arbitrage(points: &[SurfacePoint]) -> Vec<String> {
    let mut violations = Vec::new();

    for (idx, left) in points.iter().enumerate() {
        for right in points.iter().skip(idx + 1) {
            if (left.strike - right.strike).abs() < 1e-9
                && right.maturity_years > left.maturity_years
                && right.iv + 1e-9 < left.iv
            {
                violations.push(format!(
                    "calendar violation at strike {}: short={} long={}",
                    left.strike, left.iv, right.iv
                ));
            }
        }
    }

    violations
}

pub fn put_call_parity_gap(
    call_price: f64,
    put_price: f64,
    spot: f64,
    strike: f64,
    rate: f64,
    maturity_years: f64,
) -> f64 {
    (call_price - put_price) - (spot - strike * (-rate * maturity_years).exp())
}

fn d1d2(spot: f64, strike: f64, t: f64, rate: f64, vol: f64) -> (f64, f64) {
    let sqrt_t = t.sqrt();
    let d1 = ((spot / strike).ln() + (rate + 0.5 * vol * vol) * t) / (vol * sqrt_t);
    let d2 = d1 - vol * sqrt_t;
    (d1, d2)
}

fn norm_pdf(value: f64) -> f64 {
    (-(value * value) / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

fn norm_cdf(value: f64) -> f64 {
    0.5 * (1.0 + erf(value / std::f64::consts::SQRT_2))
}

fn erf(value: f64) -> f64 {
    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    let x = value.abs();

    let a1 = 0.254_829_592;
    let a2 = -0.284_496_736;
    let a3 = 1.421_413_741;
    let a4 = -1.453_152_027;
    let a5 = 1.061_405_429;
    let p = 0.327_591_1;

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * (-(x * x)).exp());

    sign * y
}
