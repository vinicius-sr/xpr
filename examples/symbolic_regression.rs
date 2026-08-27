use xpr::{Callable, Interpreter, MethodInfo};

const NUM_INPUTS: usize = 3;
const INVALID_FITNESS: f64 = f64::NEG_INFINITY;

#[derive(Debug)]
enum MethodId {
    X,
}

fn find(name: &str) -> Option<MethodInfo<MethodId>> {
    match name {
        "x" => Some(MethodInfo::new(MethodId::X, 1)),
        _ => None,
    }
}

struct Math<'a> {
    rows: &'a [[f64; NUM_INPUTS]],
    pointer: usize,
}

impl<'a> Math<'a> {
    fn new(rows: &'a [[f64; NUM_INPUTS]]) -> Self {
        Self { rows, pointer: 0 }
    }

    fn next(&mut self) {
        self.pointer += 1
    }
}

impl Callable<MethodId> for Math<'_> {
    fn call(&self, op: &MethodId, args: &[f64]) -> f64 {
        match (op, args) {
            (MethodId::X, &[i]) => self
                .rows
                .get(self.pointer)
                .and_then(|row| row.get((i - 1.0) as usize))
                .copied()
                .unwrap_or(f64::NAN),
            _ => f64::NAN,
        }
    }
}

fn evaluate_expression(phenotype: &str, rows: &[[f64; NUM_INPUTS]], targets: &[f64]) -> f64 {
    let mut math = Math::new(rows);
    let mut program = match Interpreter::compile(phenotype, find) {
        Ok(program) => program,
        Err(_) => return INVALID_FITNESS,
    };

    let mut sum_sq = 0.0;
    for expected in targets.iter() {
        match program.run(&math) {
            Ok(value) if value.is_finite() => sum_sq += (value - expected).powi(2),
            _ => return INVALID_FITNESS,
        }
        math.next();
    }

    -(sum_sq / rows.len() as f64)
}

fn main() {
    let rows = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let targets = [6.0, 15.0, 24.0];

    println!("x(1) + x(2) + x(3): {}", evaluate_expression("x(1) + x(2) + x(3)", &rows, &targets));
    println!("x(1) * x(2):        {}", evaluate_expression("x(1) * x(2)", &rows, &targets));
    println!("x(9):               {}", evaluate_expression("x(9)", &rows, &targets));
    println!("bogus(1):           {}", evaluate_expression("bogus(1)", &rows, &targets));
}
