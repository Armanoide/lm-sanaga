use mlx_rs::Array;
use std::fmt::Write;

#[derive(Debug, Default)]
pub enum PrintValueMode {
    #[default]
    None,
    Decimal(usize),
    Rounded(usize),
}

#[derive(Debug, Default)]
pub struct PrintDebugOptions {
    print_value_mode: Option<PrintValueMode>,
    head_tail: Option<usize>,
}

impl PrintDebugOptions {
    pub fn with_head_tail(mut self, head_tail: usize) -> Self {
        self.head_tail = Some(head_tail);
        self
    }

    pub fn with_print_value_mode(mut self, print_value_mode: PrintValueMode) -> Self {
        self.print_value_mode = Some(print_value_mode);
        self
    }
}

fn print_value<T: std::fmt::LowerExp + std::fmt::Debug + std::fmt::Display>(
    print_debug_options: &PrintDebugOptions,
    val: &T,
    out: &mut String,
) {
    let r = match print_debug_options.print_value_mode {
        Some(PrintValueMode::Decimal(n_decimal)) => {
            write!(out, "{:>9.*e}", n_decimal, val)
        }
        Some(PrintValueMode::Rounded(n_rounded)) => {
            write!(out, "{:>9.*}", n_rounded, val)
        }
        _ => {
            write!(out, "{:?}", val)
        }
    };
    if r.is_err() {
        eprintln!("Error formatting value: {:?}", val);
    }
}

pub fn format_debug_weight<
    T: mlx_rs::ArrayElement + std::fmt::LowerExp + std::fmt::Debug + std::fmt::Display,
>(
    array: &Array,
    print_value_mode: Option<PrintDebugOptions>,
) -> Result<String, std::fmt::Error> {
    let slice = array.as_slice::<T>();
    let shape: Vec<usize> = array.shape().iter().map(|&s| s as usize).collect();
    let print_value_mode = print_value_mode.unwrap_or_default();
    let mut out = String::new();

    write!(out, "tensor([ ")?;

    fn print_recursive<T: std::fmt::LowerExp + std::fmt::Debug + std::fmt::Display>(
        data: &[T],
        shape: &[usize],
        indent: usize,
        print_value_mode: &PrintDebugOptions,
        out: &mut String,
    ) -> Result<(), std::fmt::Error> {
        if shape.len() == 1 {
            write!(out, "[")?;
            let n = data.len();
            if let Some(k) = print_value_mode.head_tail {
                let k = k.min(n / 2);
                for (i, val) in data.iter().take(k).enumerate() {
                    print_value::<T>(print_value_mode, val, out);
                    if i != k - 1 {
                        write!(out, ", ")?;
                    }
                }
                if n > 2 * k {
                    write!(out, ", ..., ")?;
                }
                for (i, val) in data.iter().rev().take(k).rev().enumerate() {
                    if n > 2 * k || i != 0 {
                        write!(out, ", ")?;
                    }
                    print_value::<T>(print_value_mode, val, out);
                }
            } else {
                for (i, val) in data.iter().enumerate() {
                    print_value::<T>(print_value_mode, val, out);
                    if i != n - 1 {
                        write!(out, ", ")?;
                    }
                }
            }
            write!(out, "]")?;
        } else {
            let stride = data.len() / shape[0];
            write!(out, "[")?;

            let head_tail = print_value_mode.head_tail.unwrap_or(usize::MAX);
            for i in 0..shape[0] {
                if i != 0
                    && (i > 0 && (head_tail == usize::MAX)
                        || (i - 1 < head_tail || i - 1 >= shape[0] - head_tail))
                {
                    print!(",\n{:indent$}", "", indent = indent + 1);
                }

                if head_tail == usize::MAX {
                    print_recursive(
                        &data[i * stride..(i + 1) * stride],
                        &shape[1..],
                        indent + 1,
                        print_value_mode,
                        out,
                    )?;
                } else if i < head_tail || i >= shape[0] - head_tail {
                    print_recursive(
                        &data[i * stride..(i + 1) * stride],
                        &shape[1..],
                        indent + 1,
                        print_value_mode,
                        out,
                    )?;
                } else if i == head_tail {
                    write!(out, "...")?;
                }
            }
            write!(out, "]")?;
        }
        Ok(())
    }

    print_recursive(slice, shape.as_ref(), 0, &print_value_mode, &mut out)?;
    write!(out, "{}", format!("], dtype={:?})\n", array.dtype()))?;
    Ok(out.to_string())
}

pub fn print_debug_weight<
    T: mlx_rs::ArrayElement + std::fmt::LowerExp + std::fmt::Debug + std::fmt::Display,
>(
    array: &Array,
    print_value_mode: Option<PrintDebugOptions>,
) {
    print!(
        "{}",
        format_debug_weight::<T>(array, print_value_mode).unwrap_or_default()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_decimal() {
        let opts = PrintDebugOptions::default().with_print_value_mode(PrintValueMode::Decimal(3));

        let out = format_debug_weight::<f32>(
            &Array::from_slice(&[std::f32::consts::PI], &[1]),
            Some(opts),
        )
        .unwrap();

        assert!(out.contains("3.142e0")); // exponential notation with 3 decimals
    }

    #[test]
    fn test_format_value_rounded() {
        let opts = PrintDebugOptions::default().with_print_value_mode(PrintValueMode::Rounded(2));

        let out = format_debug_weight::<f32>(
            &Array::from_slice(&[std::f32::consts::PI], &[1]),
            Some(opts),
        )
        .unwrap();

        assert_eq!(out, "tensor([ [     3.14]], dtype=Float32)\n"); // right-aligned, 2 decimals
    }

    #[test]
    fn test_format_value_default_debug() {
        let opts = PrintDebugOptions::default();

        let out = format_debug_weight::<i32>(&Array::from_slice(&[42], &[1]), Some(opts)).unwrap();

        assert_eq!(out, "tensor([ [42]], dtype=Int32)\n");
    }

    #[test]
    fn test_format_debug_weight_small_array() {
        let arr = Array::from_slice(&[1.0f32, 2.0, 3.0, 4.0], &[2, 2]);

        let s = format_debug_weight::<f32>(&arr, None).unwrap();
        println!("{}", s);
        assert!(s.contains("tensor(["));
        assert!(s.contains("dtype"));
        assert!(s.contains("1"));
        assert!(s.contains("4"));
    }

    #[test]
    fn test_format_debug_weight_head_tail() {
        let arr = Array::from_slice(&*(0..20).map(|x| x as f32).collect::<Vec<_>>(), &[20]);

        let opts = PrintDebugOptions::default().with_head_tail(2);

        let s = format_debug_weight::<f32>(&arr, Some(opts)).unwrap();

        // check that it truncates
        assert!(s.contains("...,"));
        assert!(s.contains("0"));
        assert!(s.contains("19"));
    }
}
