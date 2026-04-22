use crate::{
    error::{DomainError, EngineError},
    expr::ast::{BinaryOp, Expr, UnaryOp},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dual {
    pub value: f64,
    pub derivative: f64,
}

impl Dual {
    pub fn variable(value: f64) -> Self {
        Self {
            value,
            derivative: 1.0,
        }
    }

    pub fn constant(value: f64) -> Self {
        Self {
            value,
            derivative: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaylorJet {
    coeffs: Vec<f64>,
}

impl TaylorJet {
    pub fn constant(value: f64, degree: usize) -> Self {
        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = value;
        Self { coeffs }
    }

    pub fn variable(center: f64, degree: usize) -> Self {
        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = center;
        if degree >= 1 {
            coeffs[1] = 1.0;
        }
        Self { coeffs }
    }

    pub fn degree(&self) -> usize {
        self.coeffs.len().saturating_sub(1)
    }

    pub fn coeffs(&self) -> &[f64] {
        &self.coeffs
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            coeffs: self
                .coeffs
                .iter()
                .zip(other.coeffs.iter())
                .map(|(left, right)| left + right)
                .collect(),
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            coeffs: self
                .coeffs
                .iter()
                .zip(other.coeffs.iter())
                .map(|(left, right)| left - right)
                .collect(),
        }
    }

    pub fn neg(&self) -> Self {
        Self {
            coeffs: self.coeffs.iter().map(|value| -value).collect(),
        }
    }

    pub fn scale(&self, scalar: f64) -> Self {
        Self {
            coeffs: self.coeffs.iter().map(|value| value * scalar).collect(),
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        let degree = self.degree();
        let mut coeffs = vec![0.0; degree + 1];

        for n in 0..=degree {
            let mut sum = 0.0;
            for k in 0..=n {
                sum += self.coeffs[k] * other.coeffs[n - k];
            }
            coeffs[n] = sum;
        }

        Self { coeffs }
    }

    pub fn reciprocal(&self) -> Result<Self, EngineError> {
        let degree = self.degree();
        if self.coeffs[0].abs() < 1e-14 {
            return Err(DomainError::new("division by zero in Taylor series").into());
        }

        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = 1.0 / self.coeffs[0];

        for n in 1..=degree {
            let mut sum = 0.0;
            for k in 1..=n {
                sum += self.coeffs[k] * coeffs[n - k];
            }
            coeffs[n] = -sum / self.coeffs[0];
        }

        Ok(Self { coeffs })
    }

    pub fn div(&self, other: &Self) -> Result<Self, EngineError> {
        Ok(self.mul(&other.reciprocal()?))
    }

    pub fn powi(&self, exponent: i32) -> Result<Self, EngineError> {
        if exponent == 0 {
            return Ok(Self::constant(1.0, self.degree()));
        }

        if exponent < 0 {
            return self.powi(-exponent)?.reciprocal();
        }

        let mut result = Self::constant(1.0, self.degree());
        let mut base = self.clone();
        let mut power = exponent as u32;

        while power > 0 {
            if power & 1 == 1 {
                result = result.mul(&base);
            }
            power >>= 1;
            if power > 0 {
                base = base.mul(&base);
            }
        }

        Ok(result)
    }

    pub fn derivative(&self) -> Self {
        let degree = self.degree();
        let mut coeffs = vec![0.0; degree + 1];
        for n in 1..=degree {
            coeffs[n - 1] = n as f64 * self.coeffs[n];
        }
        Self { coeffs }
    }

    pub fn integral(&self, constant: f64) -> Self {
        let degree = self.degree();
        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = constant;
        for n in 0..degree {
            coeffs[n + 1] = self.coeffs[n] / (n + 1) as f64;
        }
        Self { coeffs }
    }

    pub fn exp(&self) -> Self {
        let degree = self.degree();
        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = self.coeffs[0].exp();

        for n in 1..=degree {
            let mut sum = 0.0;
            for k in 1..=n {
                sum += k as f64 * self.coeffs[k] * coeffs[n - k];
            }
            coeffs[n] = sum / n as f64;
        }

        Self { coeffs }
    }

    pub fn ln(&self) -> Result<Self, EngineError> {
        if self.coeffs[0] <= 0.0 {
            return Err(DomainError::new("ln requires a positive value at the Taylor center").into());
        }

        let degree = self.degree();
        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = self.coeffs[0].ln();

        for n in 1..=degree {
            let mut sum = 0.0;
            for k in 1..n {
                sum += k as f64 * coeffs[k] * self.coeffs[n - k];
            }
            coeffs[n] = (n as f64 * self.coeffs[n] - sum) / (n as f64 * self.coeffs[0]);
        }

        Ok(Self { coeffs })
    }

    pub fn sqrt(&self) -> Result<Self, EngineError> {
        if self.coeffs[0] <= 0.0 {
            return Err(DomainError::new("sqrt requires a positive value at the Taylor center").into());
        }

        let degree = self.degree();
        let mut coeffs = vec![0.0; degree + 1];
        coeffs[0] = self.coeffs[0].sqrt();

        for n in 1..=degree {
            let mut sum = self.coeffs[n];
            for k in 1..n {
                sum -= coeffs[k] * coeffs[n - k];
            }
            coeffs[n] = sum / (2.0 * coeffs[0]);
        }

        Ok(Self { coeffs })
    }

    pub fn sin_cos(&self) -> (Self, Self) {
        let degree = self.degree();
        let mut sin_coeffs = vec![0.0; degree + 1];
        let mut cos_coeffs = vec![0.0; degree + 1];
        sin_coeffs[0] = self.coeffs[0].sin();
        cos_coeffs[0] = self.coeffs[0].cos();

        for n in 1..=degree {
            let mut sin_sum = 0.0;
            let mut cos_sum = 0.0;
            for k in 1..=n {
                sin_sum += k as f64 * self.coeffs[k] * cos_coeffs[n - k];
                cos_sum += k as f64 * self.coeffs[k] * sin_coeffs[n - k];
            }
            sin_coeffs[n] = sin_sum / n as f64;
            cos_coeffs[n] = -cos_sum / n as f64;
        }

        (Self { coeffs: sin_coeffs }, Self { coeffs: cos_coeffs })
    }

    pub fn atan(&self) -> Result<Self, EngineError> {
        let denominator = Self::constant(1.0, self.degree()).add(&self.mul(self));
        let derivative = self.derivative().div(&denominator)?;
        Ok(derivative.integral(self.coeffs[0].atan()))
    }

    pub fn asin(&self) -> Result<Self, EngineError> {
        if !(-1.0..=1.0).contains(&self.coeffs[0]) {
            return Err(DomainError::new("asin requires input in [-1, 1] at the Taylor center").into());
        }

        let denominator = Self::constant(1.0, self.degree())
            .sub(&self.mul(self))
            .sqrt()?;
        let derivative = self.derivative().div(&denominator)?;
        Ok(derivative.integral(self.coeffs[0].asin()))
    }

    pub fn acos(&self) -> Result<Self, EngineError> {
        if !(-1.0..=1.0).contains(&self.coeffs[0]) {
            return Err(DomainError::new("acos requires input in [-1, 1] at the Taylor center").into());
        }

        let denominator = Self::constant(1.0, self.degree())
            .sub(&self.mul(self))
            .sqrt()?;
        let derivative = self.derivative().div(&denominator)?.neg();
        Ok(derivative.integral(self.coeffs[0].acos()))
    }
}

pub fn evaluate_taylor(expr: &Expr, center: f64, degree: usize) -> Result<TaylorJet, EngineError> {
    match expr {
        Expr::Literal(value) => Ok(TaylorJet::constant(*value, degree)),
        Expr::Variable => Ok(TaylorJet::variable(center, degree)),
        Expr::ConstantPi => Ok(TaylorJet::constant(core::f64::consts::PI, degree)),
        Expr::ConstantE => Ok(TaylorJet::constant(core::f64::consts::E, degree)),
        Expr::Unary { op, expr } => {
            let value = evaluate_taylor(expr, center, degree)?;
            match op {
                UnaryOp::Plus => Ok(value),
                UnaryOp::Minus => Ok(value.neg()),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = evaluate_taylor(left, center, degree)?;
            let right = evaluate_taylor(right, center, degree)?;
            match op {
                BinaryOp::Add => Ok(left.add(&right)),
                BinaryOp::Subtract => Ok(left.sub(&right)),
                BinaryOp::Multiply => Ok(left.mul(&right)),
                BinaryOp::Divide => left.div(&right),
                BinaryOp::Power => evaluate_power_series(&left, &right),
            }
        }
        Expr::FunctionCall { name, argument } => {
            let argument = evaluate_taylor(argument, center, degree)?;
            evaluate_function_series(name, &argument)
        }
    }
}

fn evaluate_power_series(base: &TaylorJet, exponent: &TaylorJet) -> Result<TaylorJet, EngineError> {
    if exponent.coeffs()[1..].iter().all(|value| value.abs() < 1e-12) {
        let exponent_value = exponent.coeffs()[0];
        if (exponent_value - exponent_value.round()).abs() < 1e-10 {
            return base.powi(exponent_value.round() as i32);
        }
    }

    if base.coeffs()[0] <= 0.0 {
        return Err(DomainError::new(
            "non-integer power series require a positive base at the Taylor center",
        )
        .into());
    }

    Ok(base.ln()?.mul(exponent).exp())
}

fn evaluate_function_series(name: &str, argument: &TaylorJet) -> Result<TaylorJet, EngineError> {
    match name {
        "sin" => Ok(argument.sin_cos().0),
        "cos" => Ok(argument.sin_cos().1),
        "tan" => {
            let (sin, cos) = argument.sin_cos();
            sin.div(&cos)
        }
        "asin" => argument.asin(),
        "acos" => argument.acos(),
        "atan" => argument.atan(),
        "exp" => Ok(argument.exp()),
        "ln" => argument.ln(),
        "log" => Ok(argument.ln()?.scale(1.0 / 10.0_f64.ln())),
        "sqrt" => argument.sqrt(),
        "abs" => {
            if argument.coeffs()[0] > 0.0 {
                Ok(argument.clone())
            } else if argument.coeffs()[0] < 0.0 {
                Ok(argument.neg())
            } else {
                Err(DomainError::new("abs is not differentiable at zero for Taylor expansion").into())
            }
        }
        _ => Err(EngineError::NotImplemented("unsupported Taylor series function")),
    }
}
